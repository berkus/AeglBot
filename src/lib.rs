#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate dotenv;
extern crate rss;
extern crate serde_json;
extern crate telegram_bot;
#[macro_use]
extern crate diesel_derives_extra;
extern crate diesel_derives_traits;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

pub mod commands;
pub mod models;
pub mod schema;
pub mod services;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

// TODO: Implement BotCommands, make them register with bot?
pub struct Bot;

impl Bot {
    // pub fn register_catchall...

    pub fn register_command() {}
}
