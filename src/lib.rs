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
use futures::{Future, Stream};
use r2d2::Pool;
use std::env;
use telebot::{functions::*, RcBot};

pub mod commands;
pub mod datetime;
pub mod models;
pub mod schema;
pub mod services;

pub type DbConnection = LoggingConnection<PgConnection>;
pub type DbConnPool = Pool<diesel::r2d2::ConnectionManager<DbConnection>>;

pub trait BotCommand {
    /// Return command prefix to match.
    fn prefix(&self) -> &'static str;
    /// Return command description.
    fn description(&self) -> &'static str;
    /// Execute matched command.
    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        command: Option<String>,
        text: Option<String>,
    );
}

pub struct Bot {
    bot: RcBot,
    bot_name: String,
    commands: Vec<Box<BotCommand>>,
    connection_pool: DbConnPool,
}

impl Bot {
    // Public API

    pub fn new(name: &str, handle: tokio_core::reactor::Handle, token: &str) -> Self {
        Bot {
            bot: RcBot::new(handle, token).update_interval(200),
            bot_name: name.to_string(),
            commands: vec![],
            connection_pool: Self::establish_connection(),
        }
    }

    pub fn register_catchall(cmd: Box<BotCommand>) {}

    pub fn register_command(cmd: Box<BotCommand>) {
        // insert into commands while maintaining certain property:
        // - if command is a prefix of another inserted command, it must be inserted after
        //   that command.
        // - otherwise the command is inserted to the very beginning of vector.
    }

    pub fn process_messages<'a>(&'a self) -> impl Stream<Item = (), Error = failure::Error> + 'a {
        self.bot
            .get_stream()
            .filter_map(|(bot, update)| update.message.map(|msg| (bot, msg)))
            .and_then(move |(_, message)| {
                debug!("{:#?}", message);

                self.process_message(&message);

                Ok(()) // @todo return (RcBot, telebot::objects::Update) here?
            })
    }

    pub fn spawn<F>(&self, f: F)
    where
        F: Future<Item = (), Error = ()> + 'static,
    {
        self.bot.inner.handle.spawn(f);
    }

    // Internal helpers

    pub fn process_message(&self, message: &telebot::objects::Message) {
        for cmd in self.commands {
            if let (Some(cmdname), text) =
                Self::match_command(message, cmd.prefix(), &self.bot_name)
            {
                cmd.execute(&self, message, Some(cmdname), text);
            }
        }

        // if let (Some(_), text) = Self::match_command(&message, "whois", &bot_name) {
        //     WhoisCommand::execute(&bot, message, None, text, &connection);
        // } else if let (Some(_), text) = Self::match_command(&message, "psn", &bot_name) {
        //     PsnCommand::execute(&bot, message, None, text, &connection);
        // } else if let (Some(_), text) = Self::match_command(&message, "join", &bot_name) {
        //     JoinCommand::execute(&bot, message, None, text, &connection);
        // } else if let (Some(_), text) = Self::match_command(&message, "cancel", &bot_name) {
        //     CancelCommand::execute(&bot, message, None, text, &connection);
        // } else if let (Some(_), text) = Self::match_command(&message, "list", &bot_name) {
        //     ListCommand::execute(&bot, message, None, text, &connection);
        // } else if let (Some(_), text) = Self::match_command(&message, "lfg", &bot_name) {
        //     LfgCommand::execute(&bot, message, None, text, &connection);
        // } else if let (Some(_), text) = Self::match_command(&message, "details", &bot_name) {
        //     DetailsCommand::execute(&bot, message, None, text, &connection);
        // } else if let (Some(_), text) = Self::match_command(&message, "activities", &bot_name) {
        //     ActivitiesCommand::execute(&bot, message, None, text, &connection);
        // } else if let (Some(_), text) = Self::match_command(&message, "help", &bot_name) {
        //     HelpCommand::execute(&bot, message, None, text, &connection);
        // }
    }

    pub fn establish_connection() -> DbConnPool {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = diesel::r2d2::ConnectionManager::new(database_url.clone());

        r2d2::Pool::builder()
            .min_idle(Some(1))
            .max_size(15)
            .build(manager)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }

    pub fn connection(
        &self,
    ) -> r2d2::PooledConnection<diesel::r2d2::ConnectionManager<DbConnection>> {
        self.connection_pool.get().unwrap()
    }

    pub fn spawn_message(&self, m: telebot::functions::WrapperMessage) {
        self.bot
            .inner
            .handle
            .spawn(m.send().map(|_| ()).map_err(|e| error!("Error: {:?}", e)));
    }

    pub fn send_plain_reply(&self, source: &telebot::objects::Message, t: String) {
        self.spawn_message(
            self.bot
                .message(source.chat.id, t)
                .reply_to_message_id(source.message_id)
                .disable_notificaton(true)
                .disable_web_page_preview(true),
        );
    }

    pub fn send_html_reply(&self, source: &telebot::objects::Message, t: String) {
        self.spawn_message(
            self.bot
                .message(source.chat.id, t)
                .reply_to_message_id(source.message_id)
                .parse_mode(ParseMode::HTML)
                .disable_notificaton(true)
                .disable_web_page_preview(true),
        );
    }

    pub fn send_html_message(&self, chat: telebot::objects::Integer, t: String) {
        self.spawn_message(
            self.bot
                .message(chat, t)
                .parse_mode(ParseMode::HTML)
                .disable_notificaton(true)
                .disable_web_page_preview(true),
        );
    }

    /// Match command in both variations (with bot name and without bot name).
    /// @param data Input text received from Telegram.
    /// @param command Command name without leading slash.
    /// @param bot_name Registered bot name.
    /// @returns A pair of matched command and remainder of the message text.
    /// (None, None) if command did not match,
    /// (command, and Some remaining text after command otherwise).
    ///
    /// @todo Some clients send /cancel593@AeglBot on click, so probably need to match longest
    /// command prefix if the bot name also matches in command
    /// (basically, if ends_with "@BotName", strip it off and match command prefixes)
    fn match_command(
        msg: &telebot::objects::Message,
        command: &str,
        bot_name: &str,
    ) -> (Option<String>, Option<String>) {
        if msg.text.is_none() {
            return (None, None);
        }

        let data = msg.text.as_ref().unwrap();
        debug!("matching text {:#?}", data);

        let command = "/".to_owned() + command;
        let long_command = format!("{}@{}", command, bot_name);
        debug!("matching {:#?} against {:#?}", data, long_command);
        if data.starts_with(&long_command) {
            debug!(".. matched");
            return (
                Some(long_command.clone()),
                data.get(long_command.len()..)
                    .map(|x| x.trim_left().to_string())
                    .filter(|y| !y.is_empty()),
            );
        }
        debug!("matching {:#?} against {:#?}", data, command);
        if data.starts_with(&command) {
            debug!(".. matched");
            return (
                Some(command.clone()),
                data.get(command.len()..)
                    .map(|x| x.trim_left().to_string())
                    .filter(|y| !y.is_empty()),
            );
        }
        (None, None)
    }
}
