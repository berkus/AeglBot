#![feature(crate_in_paths, extern_prelude)] // features from edition-2018
#![allow(proc_macro_derive_resolution_fallback)] // see https://github.com/rust-lang/rust/issues/50504
#![allow(unused_imports)] // during development

extern crate r2d2;
#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate chrono_english;
extern crate chrono_tz;
extern crate diesel_logger;
extern crate dotenv;
extern crate rss;
extern crate serde_json;
extern crate telebot;
#[macro_use]
extern crate diesel_derives_extra;
extern crate diesel_derives_traits;
extern crate failure;
extern crate futures;
#[macro_use]
extern crate log;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_logger::LoggingConnection;
use dotenv::dotenv;
use r2d2::Pool;
use std::env;

pub mod commands;
pub mod datetime;
pub mod models;
pub mod schema;
pub mod services;

pub type DbConnection = LoggingConnection<PgConnection>;

pub fn establish_connection() -> Pool<diesel::r2d2::ConnectionManager<DbConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = diesel::r2d2::ConnectionManager::new(database_url.clone());

    r2d2::Pool::builder()
        .min_idle(Some(1))
        .max_size(15)
        .build(manager)
        .expect(&format!("Error connecting to {}", database_url))
}

// TODO: Implement BotCommands, make them register with bot?
pub struct Bot;

impl Bot {
    // pub fn register_catchall...

    pub fn register_command() {}
}
