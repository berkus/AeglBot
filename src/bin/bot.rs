// Parallel Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (yeah, i'd prefer native, although there are ways to use natty through jlink
// or take python equivalent from https://dateparser.readthedocs.io/en/latest/)
extern crate aegl_bot;
extern crate diesel;
extern crate dotenv;
extern crate futures;
extern crate rss;
extern crate telegram_bot;
extern crate tokio_core;

use aegl_bot::commands::*;
use aegl_bot::models::*;
use diesel::prelude::*;
use dotenv::dotenv;
use futures::Stream;
use std::env;
use telegram_bot::*;
use tokio_core::reactor::Core;

/// Match command in both variations (with bot name and without bot name).
/// @param data Input text received from Telegram.
/// @param command Command name without leading slash.
/// @param bot_name Registered bot name.
/// @returns None if command did not match, Some remaining text after command otherwise.
fn match_command(data: &str, command: &str, bot_name: &str) -> Option<String> {
    let command = "/".to_owned() + &command;
    let long_command = format!("{}@{}", command, bot_name);
    if data.starts_with(&long_command) {
        return Some(data[long_command.len() + 1..].to_string());
    }
    if data.starts_with(&command) {
        return Some(data[command.len() + 1..].to_string());
    }
    None
}

fn main() {
    dotenv().ok();

    use aegl_bot::schema::activities::dsl::*;
    use aegl_bot::schema::alerts::dsl::*;
    use aegl_bot::schema::guardians::dsl::*;

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

    let results2 = activities
        .load::<Activity>(&connection)
        .expect("Error loading activities");

    println!("Displaying {} activities", results2.len());
    for act in results2 {
        println!("{}", act.format_name());
    }

    let results3 = alerts
        .limit(5)
        .load::<Alert>(&connection)
        .expect("Error loading alerts");

    println!("Displaying {} alerts", results3.len());
    for alrt in results3 {
        println!("{}", alrt.title);
    }

    let guar = guardians
        .find(1)
        .first::<Guardian>(&connection)
        .expect("Guardian with id 1 not found");
    let results4 = PlannedActivity::belonging_to(&guar)
        .load::<PlannedActivity>(&connection)
        .expect("Error loading activities");

    println!("Displaying {} planned activities", results4.len());
    for act in results4 {
        println!("{}", act);
    }

    let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");

    let mut core = Core::new().unwrap();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let api = Api::configure(token)
        .build(core.handle())
        .expect("Telegram API connect failed");

    // Fetch new updates via long poll method
    let future = api.stream().for_each(|update| {
        // If the received update contains a new message...
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);

                // Plug awesome-bot style routing in here
                if let Some(text) = match_command(data, "whois", &bot_name) {
                    WhoisCommand::handle(&api, &message, &text, &connection);
                } else if let Some(text) = match_command(data, "psn", &bot_name) {
                    PsnCommand::handle(&api, &message, &text, &connection);
                }
            }
        }

        Ok(())
    });

    // fixedRateTimer(name = "Alerts", daemon = true, initialDelay = 0, period = 60*1000 /* millis */) {
    //     log.info("alerts check")
    //     AlertsWatcher(store).check(wfChatId, this@AeglBot)
    // }

    core.run(future).unwrap();
}
