// Parallel Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (yeah, i'd prefer native, although there are ways to use natty through jlink
// or take python equivalent from https://dateparser.readthedocs.io/en/latest/)
#![feature(futures_api, async_await, await_macro, extern_prelude)]

extern crate aegl_bot;
extern crate diesel;
extern crate dotenv;
extern crate futures;
extern crate r2d2;
extern crate rss;
extern crate telebot;
extern crate tokio;
extern crate tokio_core;
#[macro_use]
extern crate log;
extern crate fern;

use aegl_bot::commands::*;
use aegl_bot::services::*;
use dotenv::dotenv;
use futures::{Future, IntoFuture, Stream};
use std::env;
use std::time::{Duration, Instant};
use telebot::RcBot;
use tokio::timer::Interval;
use tokio_core::reactor::Core;

/// Match command in both variations (with bot name and without bot name).
/// @param data Input text received from Telegram.
/// @param command Command name without leading slash.
/// @param bot_name Registered bot name.
/// @returns A pair of matched command and remainder of the message text.
/// (None, None) if command did not match,
/// (command, and Some remaining text after command otherwise).
fn match_command(
    msg: &telebot::objects::Message,
    command: &str,
    bot_name: &str,
) -> (Option<String>, Option<String>) {
    if let None = msg.text {
        return (None, None);
    }

    let data = msg.text.as_ref().unwrap();
    debug!("matching text {:#?}", data);

    let command = "/".to_owned() + &command;
    let long_command = format!("{}@{}", command, bot_name);
    debug!("matching {:#?} against {:#?}", data, long_command);
    if data.starts_with(&long_command) {
        debug!(".. matched");
        return (
            Some(long_command.clone()),
            data.get(long_command.len()..)
                .map(|x| x.trim_left().to_string())
                .filter(|y| y.len() != 0),
        );
    }
    debug!("matching {:#?} against {:#?}", data, command);
    if data.starts_with(&command) {
        debug!(".. matched");
        return (
            Some(command.clone()),
            data.get(command.len()..)
                .map(|x| x.trim_left().to_string())
                .filter(|y| y.len() != 0),
        );
    }
    (None, None)
}

fn setup_logging() -> Result<(), fern::InitError> {
    use fern::colors::{Color, ColoredLevelConfig};

    // Color setup from fern examples
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);
    let colors_level = colors_line.clone().info(Color::Green);

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
            .open(format!("logs/bot-{}.log", chrono::Local::now().format("%Y%m%d-%H%M%S")))?,
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

    let connection_pool = aegl_bot::establish_connection();

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
        let bot = RcBot::new(core.handle(), &token).update_interval(200);

        // WhoisCommand::register(&api, &connection);
        // PsnCommand::register(&api, &connection);

        let stream = bot
            .get_stream()
            .filter_map(|(bot, update)| update.message.map(|msg| (bot, msg)))
            .and_then(|(_, message)| {
                debug!("{:#?}", message);

                let connection = connection_pool.get().unwrap();

                // @todo Plug awesome-bot style routing in here
                if let (Some(_), text) = match_command(&message, "whois", &bot_name) {
                    WhoisCommand::execute(&bot, message, None, text, &connection);
                } else if let (Some(_), text) = match_command(&message, "psn", &bot_name) {
                    PsnCommand::execute(&bot, message, None, text, &connection);
                } else if let (Some(_), text) = match_command(&message, "join", &bot_name) {
                    JoinCommand::execute(&bot, message, None, text, &connection);
                } else if let (Some(_), text) = match_command(&message, "cancel", &bot_name) {
                    CancelCommand::execute(&bot, message, None, text, &connection);
                } else if let (Some(_), text) = match_command(&message, "list", &bot_name) {
                    ListCommand::execute(&bot, message, None, text, &connection);
                } else if let (Some(_), text) = match_command(&message, "lfg", &bot_name) {
                    LfgCommand::execute(&bot, message, None, text, &connection);
                } else if let (Some(_), text) = match_command(&message, "details", &bot_name) {
                    DetailsCommand::execute(&bot, message, None, text, &connection);
                } else if let (Some(_), text) = match_command(&message, "activities", &bot_name) {
                    ActivitiesCommand::execute(&bot, message, None, text, &connection);
                } else if let (Some(_), text) = match_command(&message, "help", &bot_name) {
                    HelpCommand::execute(&bot, message, None, text, &connection);
                }

                Ok(())
            });

        let alerts_bot = bot.clone();
        let alerts_pool = connection_pool.clone();
        let alert_task = Interval::new(Instant::now(), Duration::from_secs(60))
            .for_each(move |_| {
                info!("alerts check");
                let connection = alerts_pool.get().unwrap();
                alerts_watcher::check(&alerts_bot, wf_alerts_chat, &connection);
                Ok(()) // @todo forward return from check()
            }).map_err(|e| panic!("Alert thread errored; err={:?}", e));

        let reminder_bot = bot.clone();
        let reminder_pool = connection_pool.clone();
        let reminder_task = Interval::new(Instant::now(), Duration::from_secs(60))
            .for_each(move |_| {
                // @todo Add a thread that would get once a minute a list of planned activities and
                // notify when the time is closing in.
                // e.g.
                // Event starting in 15 minutes: Iron Banner with @dozniak, @aero_kamero (4 more can join)
                info!("reminder check");
                let connection = reminder_pool.get().unwrap();
                reminder::check(&reminder_bot, lfg_chat, &connection);
                Ok(()) // @todo forward return from check()
            }).map_err(|e| panic!("Reminder thread errored; err={:?}", e));

        bot.inner.handle.spawn(alert_task);
        bot.inner.handle.spawn(reminder_task);

        core.run(stream.for_each(|_| Ok(())).into_future()).unwrap(); // @todo handle connection errors and restart bot after pause
    }
}

#[test]
fn test_guardians() {
    use aegl_bot::models::Guardian;
    use aegl_bot::schema::guardians::dsl::*;

    dotenv().ok();

    let connection = aegl_bot::establish_connection().get().unwrap();

    let results = guardians
        // .filter(published.eq(true))
        .limit(5)
        .load::<Guardian>(&connection)
        .expect("Error loading guardians");

    println!("Displaying {} guardians", results.len());
    for guar in results {
        println!("{}", guar);
    }
}

#[test]
fn test_activities() {
    use aegl_bot::models::Activity;
    use aegl_bot::schema::activities::dsl::*;

    dotenv().ok();

    let connection = aegl_bot::establish_connection().get().unwrap();

    let results = activities
        .load::<Activity>(&connection)
        .expect("Error loading activities");

    println!("Displaying {} activities", results.len());
    for act in results {
        println!("{}", act.format_name());
    }
}

#[test]
fn test_alerts() {
    use aegl_bot::models::Alert;
    use aegl_bot::schema::alerts::dsl::*;

    dotenv().ok();

    let connection = aegl_bot::establish_connection().get().unwrap();

    let results = alerts
        .limit(5)
        .load::<Alert>(&connection)
        .expect("Error loading alerts");

    println!("Displaying {} alerts", results.len());
    for alrt in results {
        println!("{}", alrt.title);
    }
}

#[test]
fn test_planned_activities() {
    use aegl_bot::models::{Guardian, PlannedActivity};
    use aegl_bot::schema::guardians::dsl::*;

    dotenv().ok();

    let connection = aegl_bot::establish_connection().get().unwrap();

    let guar = guardians
        .find(1)
        .first::<Guardian>(&connection)
        .expect("Guardian with id 1 not found");
    let results = PlannedActivity::belonging_to(&guar)
        .load::<PlannedActivity>(&connection)
        .expect("Error loading activities");

    println!("Displaying {} planned activities", results.len());
    for act in results {
        println!("{}", act.display(&connection, &guar));
    }
}
