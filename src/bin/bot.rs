// Async Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (yeah, i'd prefer native, although there are ways to use natty through jlink
// or take python equivalent from https://dateparser.readthedocs.io/en/latest/)
#![feature(futures_api, async_await, await_macro, nll)]

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

use aegl_bot::{
    commands::*,
    datetime::{d2_reset_time, reference_date, start_at_time, start_at_weekday_time},
    services::*,
    Bot,
};
use dotenv::dotenv;
use failure::Error;
use futures::{Future, IntoFuture, Stream};
use std::env;
use std::time::Instant;
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
        let mut bot = Bot::new(&bot_name, core.handle(), &token);

        bot.register_command(ActivitiesCommand::new());
        bot.register_command(CancelCommand::new());
        bot.register_command(EditCommand::new());
        bot.register_command(EditGuardianCommand::new());
        bot.register_command(HelpCommand::new());
        bot.register_command(InfoCommand::new());
        bot.register_command(JoinCommand::new());
        bot.register_command(LfgCommand::new());
        bot.register_command(ListCommand::new());
        bot.register_command(ManageCommand::new());
        bot.register_command(PsnCommand::new());
        bot.register_command(WhoisCommand::new());

        let stream = bot.process_messages();

        let alert_task = setup_timer_task(
            Interval::new(
                Instant::now(),
                chrono::Duration::minutes(1).to_std().unwrap(),
            ),
            bot.clone(),
            move |bot| alerts_watcher::check(bot, wf_alerts_chat),
        );

        let reminder_task = setup_timer_task(
            Interval::new(
                Instant::now(),
                chrono::Duration::minutes(1).to_std().unwrap(),
            ),
            bot.clone(),
            move |bot| reminder::check(bot, lfg_chat),
        );

        let daily_reset_task = setup_timer_task(
            Interval::new(
                start_at_time(reference_date(), d2_reset_time()),
                chrono::Duration::days(1).to_std().unwrap(),
            ),
            bot.clone(),
            move |bot| destiny2_schedule::daily_reset(bot, lfg_chat),
        );

        let weekly_reset_task = setup_timer_task(
            Interval::new(
                start_at_weekday_time(reference_date(), chrono::Weekday::Tue, d2_reset_time()),
                chrono::Duration::weeks(1).to_std().unwrap(),
            ),
            bot.clone(),
            move |bot| destiny2_schedule::major_weekly_reset(bot, lfg_chat),
        );

        let friday_reset_task = setup_timer_task(
            Interval::new(
                start_at_weekday_time(reference_date(), chrono::Weekday::Fri, d2_reset_time()),
                chrono::Duration::weeks(1).to_std().unwrap(),
            ),
            bot.clone(),
            move |bot| destiny2_schedule::minor_weekly_reset(bot, lfg_chat),
        );

        let monday_reset_task = setup_timer_task(
            Interval::new(
                start_at_weekday_time(reference_date(), chrono::Weekday::Mon, d2_reset_time()),
                chrono::Duration::weeks(1).to_std().unwrap(),
            ),
            bot.clone(),
            move |bot| destiny2_schedule::end_of_weekend(bot, lfg_chat),
        );

        bot.spawn(alert_task);
        bot.spawn(reminder_task);
        bot.spawn(daily_reset_task);
        bot.spawn(weekly_reset_task);
        bot.spawn(friday_reset_task);
        bot.spawn(monday_reset_task);

        core.run(
            stream
                .for_each(|_| Ok(()))
                .map_err(|e| error!("Caught an error {}", e))
                .into_future(),
        ).unwrap();
    }
}

/// Setup handling for timer Stream over interval `i` to run closure `f` using cloned bot `b`.
fn setup_timer_task<F>(
    interval: Interval,
    bot: Bot,
    mut fun: F,
) -> impl Future<Item = (), Error = ()>
where
    F: FnMut(&Bot) -> Result<(), Error>,
{
    interval
        .for_each(move |_| fun(&bot).or_else(|_| Ok(())))
        .map_err(|e| panic!("Daily reset thread errored; err={:?}", e))
}
