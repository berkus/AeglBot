// Parallel Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (yeah, i'd prefer native, although there are ways to use natty through jlink
// or take python equivalent from https://dateparser.readthedocs.io/en/latest/)
#![feature(futures_api, async_await, await_macro)]

extern crate aegl_bot;
extern crate diesel;
extern crate dotenv;
extern crate futures;
extern crate pretty_env_logger;
extern crate rss;
extern crate telegram_bot;
extern crate tokio;
extern crate tokio_core;
#[macro_use]
extern crate log;

use aegl_bot::commands::*;
use aegl_bot::services::*;
use diesel::prelude::*;
use dotenv::dotenv;
use futures::{Future, Stream};
use std::env;
use std::time::{Duration, Instant};
use telegram_bot::*;
use tokio::timer::Interval;
use tokio_core::reactor::Core;

/// Match command in both variations (with bot name and without bot name).
/// @param data Input text received from Telegram.
/// @param command Command name without leading slash.
/// @param bot_name Registered bot name.
/// @returns A pair of matched command and remainder of the message text.
/// (None, None) if command did not match,
/// (command, and Some remaining text after command otherwise).
fn match_command(data: &str, command: &str, bot_name: &str) -> (Option<String>, Option<String>) {
    let command = "/".to_owned() + &command;
    let long_command = format!("{}@{}", command, bot_name);
    if data.starts_with(&long_command) {
        return (
            Some(long_command.clone()),
            data.get(long_command.len()..)
                .map(|x| x.trim_left().to_string())
                .filter(|y| y.len() != 0),
        );
    }
    if data.starts_with(&command) {
        return (
            Some(command.clone()),
            data.get(command.len()..)
                .map(|x| x.trim_left().to_string())
                .filter(|y| y.len() != 0),
        );
    }
    (None, None)
}

fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let connection_pool = aegl_bot::establish_connection();

    // TimeZone.setDefault(TimeZone.getTimeZone(config.getString("bot.timezone")))
    let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
    let lfg_chat = ChatId::new(
        env::var("BOT_LFG_CHAT_ID")
            .expect("BOT_LFG_CHAT_ID must be set")
            .parse::<i64>()
            .expect("BOT_LFG_CHAT_ID must be a valid telegram chat id"),
    );
    let wf_alerts_chat = ChatId::new(
        env::var("BOT_WF_CHAT_ID")
            .expect("BOT_WF_CHAT_ID must be set")
            .parse::<i64>()
            .expect("BOT_WF_CHAT_ID must be a valid telegram chat id"),
    );

    let mut core = Core::new().unwrap();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let api = Api::configure(token)
        .build(core.handle())
        .expect("Telegram API connect failed");

    // WhoisCommand::register(&api, &connection);
    // PsnCommand::register(&api, &connection);

    // Fetch new updates via long poll method
    let future = api.stream().for_each(|update| {
        // If the received update contains a new message...
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);

                // Plug awesome-bot style routing in here
                if let (Some(_), text) = match_command(data, "whois", &bot_name) {
                    WhoisCommand::execute(&api, &message, None, text, &connection);
                } else if let (Some(_), text) = match_command(data, "psn", &bot_name) {
                    PsnCommand::execute(&api, &message, None, text, &connection);
                } else if let (Some(_), text) = match_command(data, "join", &bot_name) {
                    JoinCommand::execute(&api, &message, None, text, &connection);
                } else if let (Some(_), text) = match_command(data, "cancel", &bot_name) {
                    CancelCommand::execute(&api, &message, None, text, &connection);
                } else if let (Some(_), text) = match_command(data, "list", &bot_name) {
                    ListCommand::execute(&api, &message, None, text, &connection);
                } else if let (Some(_), text) = match_command(data, "lfg", &bot_name) {
                    LfgCommand::execute(&api, &message, None, text, &connection);
                } else if let (Some(_), text) = match_command(data, "details", &bot_name) {
                    DetailsCommand::execute(&api, &message, None, text, &connection);
                } else if let (Some(_), text) = match_command(data, "activities", &bot_name) {
                    ActivitiesCommand::execute(&api, &message, None, text, &connection);
                } else if let (Some(_), text) = match_command(data, "help", &bot_name) {
                    HelpCommand::execute(&api, &message, None, text, &connection);
                }
            }
        }

        Ok(())
    });

    // let alert_api = api.clone();
    // let alert_task = Interval::new(Instant::now(), Duration::from_secs(60))
    //     .for_each(move |_| {
    //         println!("alerts check"); // @todo Proper logger!
    //         alerts_watcher::check(&alert_api, wf_alerts_chat, &connection);
    //         Ok(())
    //     }).map_err(|e| panic!("Alert thread errored; err={:?}", e));

    // let reminder_api = api.clone();
    // let reminder_task = Interval::new(Instant::now(), Duration::from_secs(60))
    //     .for_each(move |_| {
    //         // @todo Add a thread that would get once a minute a list of planned activities and
    //         // notify when the time is closing in.
    //         // e.g.
    //         // Event starting in 15 minutes: Iron Banner with @dozniak, @aero_kamero (4 more can join)
    //         //     log.info("reminder check")
    //         //     Reminder(store).check(lfgChatId)
    //         Ok(())
    //     }).map_err(|e| panic!("Reminder thread errored; err={:?}", e));

    // tokio::spawn(alert_task);
    core.run(future).unwrap(); // @todo handle connection errors and restart bot after pause
}

#[test]
fn test_guardians() {
    use aegl_bot::models::Guardian;
    use aegl_bot::schema::guardians::dsl::*;

    dotenv().ok();

    let connection = aegl_bot::establish_connection();

    let results = guardians
        // .filter(published.eq(true))
        // .limit(5)
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

    let connection = aegl_bot::establish_connection();

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

    let connection = aegl_bot::establish_connection();

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

    let connection = aegl_bot::establish_connection();

    let guar = guardians
        .find(1)
        .first::<Guardian>(&connection)
        .expect("Guardian with id 1 not found");
    let results = PlannedActivity::belonging_to(&guar)
        .load::<PlannedActivity>(&connection)
        .expect("Error loading activities");

    println!("Displaying {} planned activities", results.len());
    for act in results {
        println!("{}", act);
    }
}
