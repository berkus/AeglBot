// Parallel Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (yeah, i'd prefer native, although there are ways to use natty through jlink
// or take python equivalent from https://dateparser.readthedocs.io/en/latest/)
#![feature(futures_api, async_await, await_macro, extern_prelude, nll)]

extern crate aegl_bot;
extern crate diesel;
extern crate dotenv;
extern crate failure;
extern crate futures;
extern crate r2d2;
extern crate rss;
extern crate telebot;
extern crate tokio;
extern crate tokio_core;
#[macro_use]
extern crate log;
extern crate fern;

use aegl_bot::{commands::*, services::*, Bot};
use dotenv::dotenv;
use failure::Error;
use futures::{Future, IntoFuture, Stream};
use std::env;
use std::time::{Duration, Instant};
use telebot::error::TelegramError;
use tokio::timer::Interval;
use tokio_core::reactor::Core;

fn setup_logging() -> Result<(), fern::InitError> {
    use fern::colors::{Color, ColoredLevelConfig};

    // Color setup from fern examples
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);
    let colors_level = colors_line.info(Color::Green);

    let console_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}{date}[{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ))
        }).level(log::LevelFilter::Info)
        .chain(std::io::stdout());

    let file_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        }).level(log::LevelFilter::Trace)
        .chain(
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false) // don't overwrite log file each run
                .open(format!(
                    "logs/bot-{}.log",
                    chrono::Local::now().format("%Y%m%d-%H%M%S")
                ))?,
        );

    fern::Dispatch::new()
        .chain(console_config)
        .chain(file_config)
        .apply()?;

    Ok(())
}

fn main() {
    dotenv().ok();
    setup_logging().expect("failed to initialize logging");

    // TimeZone.setDefault(TimeZone.getTimeZone(config.getString("bot.timezone")))
    let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
    let lfg_chat = env::var("BOT_LFG_CHAT_ID")
        .expect("BOT_LFG_CHAT_ID must be set")
        .parse::<i64>()
        .expect("BOT_LFG_CHAT_ID must be a valid telegram chat id");
    let wf_alerts_chat = env::var("BOT_WF_CHAT_ID")
        .expect("BOT_WF_CHAT_ID must be set")
        .parse::<i64>()
        .expect("BOT_WF_CHAT_ID must be a valid telegram chat id");

    let mut core = Core::new().unwrap();
    loop {
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
        let bot = Bot::new(&bot_name, core.handle(), &token);

        // bot.register_command(ActivitiesCommand);
        // bot.register_command(CancelCommand);
        // bot.register_command(DetailsCommand);
        // bot.register_command(HelpCommand);
        // bot.register_command(JoinCommand);
        // bot.register_command(LfgCommand);
        // bot.register_command(ListCommand);
        // bot.register_command(PsnCommand);
        // bot.register_command(WhoisCommand);

        let stream = bot.process_messages();

        let alerts_bot = bot.clone();
        let alert_task = Interval::new(Instant::now(), Duration::from_secs(60))
            .for_each(move |_| {
                info!("alerts check");
                alerts_watcher::check(&alerts_bot, wf_alerts_chat);
                Ok(())
            }).map_err(|e| panic!("Alert thread errored; err={:?}", e));

        let reminder_bot = bot.clone();
        let reminder_task = Interval::new(Instant::now(), Duration::from_secs(60))
            .for_each(move |_| {
                info!("reminder check");
                reminder::check(&reminder_bot, lfg_chat);
                Ok(())
            }).map_err(|e| panic!("Reminder thread errored; err={:?}", e));

        bot.spawn(alert_task);
        bot.spawn(reminder_task);

        core.run(stream.for_each(|_| Ok(())).into_future())
            .map_err(|error| match error.downcast_ref::<TelegramError>() {
                Some(message) => {
                    error!("Telegram server error: {}, restarting connection.", message);
                    std::thread::sleep(Duration::from_secs(30));
                    Ok(())
                }
                None => Err(error),
            }).unwrap();
    }
}
