#![feature(crate_in_paths, extern_prelude, nll)] // features from edition-2018
#![allow(proc_macro_derive_resolution_fallback)] // see https://github.com/rust-lang/rust/issues/50504
#![allow(unused_imports)] // during development
#![feature(slice_sort_by_cached_key)]
#![feature(type_ascription)]

extern crate r2d2;
#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate chrono_english;
extern crate chrono_tz;
extern crate diesel_logger;
extern crate dotenv;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate rss;
extern crate serde_json;
extern crate telebot;
#[macro_use]
extern crate diesel_derives_extra;
extern crate diesel_derives_traits;
extern crate failure;
extern crate futures;
extern crate futures_retry;
#[macro_use]
extern crate log;
#[cfg(target_os = "linux")]
extern crate procfs;
extern crate regex;
extern crate tokio_core;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_logger::LoggingConnection;
use dotenv::dotenv;
use failure::Error;
use futures::{Future, Stream};
use futures_retry::{RetryPolicy, StreamRetryExt};
use r2d2::Pool;
use std::time::Duration;
use std::{
    env,
    sync::{Arc, RwLock},
};
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
    /// To support sub-commands the prefix for root commands should start with '/'.
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

#[derive(Clone)]
pub struct Bot {
    bot: RcBot,
    bot_name: String,
    commands: Arc<RwLock<Vec<Box<BotCommand>>>>,
    connection_pool: DbConnPool,
}

impl Bot {
    // Public API

    pub fn new(name: &str, handle: tokio_core::reactor::Handle, token: &str) -> Self {
        Bot {
            bot: RcBot::new(handle, token).update_interval(200),
            bot_name: name.to_string(),
            commands: Arc::new(RwLock::new(Vec::new())),
            connection_pool: Self::establish_connection(),
        }
    }

    // pub fn register_catchall(cmd: Box<BotCommand>) {}

    // Insert into commands while maintaining certain property:
    // - if command is a prefix of another inserted command, it must be inserted after
    //   that command.
    // - otherwise the command is inserted to the very beginning of vector.
    // This allows correct parsing order fof commands that are prefixes of another command.
    pub fn register_command(&mut self, cmd: Box<BotCommand>) {
        let mut insertion_index = 0;
        for (idx, command) in self.commands.read().unwrap().iter().enumerate() {
            if command.prefix().starts_with(cmd.prefix()) {
                insertion_index = idx + 1;
            }
        }

        self.commands.write().unwrap().insert(insertion_index, cmd);
    }

    pub fn list_commands(&self) -> Vec<(String, String)> {
        self.commands
            .read()
            .unwrap()
            .iter()
            .fold(vec![], |mut acc, cmd| {
                acc.push((cmd.prefix().to_string(), cmd.description().to_string()));
                acc
            })
    }

    pub fn process_messages<'a>(&'a self) -> impl Stream<Item = (), Error = failure::Error> + 'a {
        self.bot
            .get_stream()
            .retry(Bot::handle_error)
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

    fn handle_error(error: Error) -> RetryPolicy<Error> {
        // count errors
        error!("handle_error");
        match error.downcast_ref::<telebot::error::Error>() {
            Some(te) => {
                error!("Telegram error: {}, retrying connection.", te);
                RetryPolicy::WaitRetry(Duration::from_secs(30))
            }
            None => {
                error!("handle_error didnt match, real error {:?}", error);
                //handle_error didnt match, real error Io(Custom { kind: Other, error: StringError("failed to lookup address information: nodename nor servname provided, or not known") })
                RetryPolicy::ForwardError(error)
            }
        }
    }

    pub fn process_message(&self, message: &telebot::objects::Message) {
        for cmd in self.commands.read().unwrap().iter() {
            if let (Some(cmdname), text) =
                Self::match_command(message, cmd.prefix(), &self.bot_name)
            {
                return cmd.execute(&self, message, Some(cmdname), text);
            }
        }
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

    pub fn send_plain_reply(&self, source: &telebot::objects::Message, text: String) {
        self.spawn_message(
            self.bot
                .message(source.chat.id, text)
                .reply_to_message_id(source.message_id)
                .disable_notificaton(true)
                .disable_web_page_preview(true),
        );
    }

    pub fn send_html_reply(&self, source: &telebot::objects::Message, text: String) {
        self.spawn_message(
            self.bot
                .message(source.chat.id, text)
                .reply_to_message_id(source.message_id)
                .parse_mode(ParseMode::HTML)
                .disable_notificaton(true)
                .disable_web_page_preview(true),
        );
    }

    pub fn send_plain_message(&self, chat: telebot::objects::Integer, text: String) {
        self.spawn_message(
            self.bot
                .message(chat, text)
                .disable_notificaton(true)
                .disable_web_page_preview(true),
        );
    }

    pub fn send_md_message(&self, chat: telebot::objects::Integer, text: String) {
        self.spawn_message(
            self.bot
                .message(chat, text)
                .parse_mode(ParseMode::Markdown)
                .disable_notificaton(true)
                .disable_web_page_preview(true),
        );
    }

    pub fn send_html_message(&self, chat: telebot::objects::Integer, text: String) {
        self.spawn_message(
            self.bot
                .message(chat, text)
                .parse_mode(ParseMode::HTML)
                .disable_notificaton(true)
                .disable_web_page_preview(true),
        );
    }

    pub fn send_html_message_with_notification(
        &self,
        chat: telebot::objects::Integer,
        text: String,
    ) {
        self.spawn_message(
            self.bot
                .message(chat, text)
                .parse_mode(ParseMode::HTML)
                .disable_notificaton(false)
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

        let command = command.to_owned();
        let non_command = format!("{}@", command);
        let long_command = format!("{}@{}", command, bot_name);

        // Some clients send /cancel593@AeglBot on click, so probably need to match longest
        // command prefix if the bot name also matches in command
        // (basically, if ends_with "@BotName", strip it off and match command prefixes)
        if data.ends_with(&format!("@{}", bot_name)) {
            let end = data.len() - bot_name.len() - 1;
            let data = &data[0..end];
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
            return (None, None);
        }

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

        if data.starts_with(&non_command) {
            debug!(".. some other bot matched");
            return (None, None);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_core::reactor::Core;

    // Command is prefix of another command.
    struct PrefixCommand;

    struct PrefixTwoCommand;

    impl PrefixCommand {
        pub fn new() -> Box<Self> {
            Box::new(PrefixCommand)
        }
    }

    impl BotCommand for PrefixCommand {
        fn prefix(&self) -> &'static str {
            "/prefix"
        }

        fn description(&self) -> &'static str {
            "Test"
        }

        fn execute(
            &self,
            _bot: &Bot,
            _message: &telebot::objects::Message,
            _command: Option<String>,
            _name: Option<String>,
        ) {
        }
    }

    impl PrefixTwoCommand {
        pub fn new() -> Box<Self> {
            Box::new(PrefixTwoCommand)
        }
    }

    impl BotCommand for PrefixTwoCommand {
        fn prefix(&self) -> &'static str {
            "/prefixtwo"
        }

        fn description(&self) -> &'static str {
            "Test two"
        }

        fn execute(
            &self,
            _bot: &Bot,
            _message: &telebot::objects::Message,
            _command: Option<String>,
            _name: Option<String>,
        ) {
        }
    }

    #[test]
    fn test_command_insertion_order1() {
        dotenv().ok();
        let core = Core::new().unwrap();
        let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
        let mut bot = Bot::new(&bot_name, core.handle(), &token);

        bot.register_command(PrefixCommand::new());
        bot.register_command(PrefixTwoCommand::new());

        assert_eq!(
            bot.list_commands(),
            vec![
                ("/prefixtwo".to_string(), "Test two".to_string()),
                ("/prefix".to_string(), "Test".to_string())
            ]
        );
    }

    #[test]
    fn test_command_insertion_order2() {
        dotenv().ok();
        let core = Core::new().unwrap();
        let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
        let mut bot = Bot::new(&bot_name, core.handle(), &token);

        bot.register_command(PrefixTwoCommand::new());
        bot.register_command(PrefixCommand::new());

        assert_eq!(
            bot.list_commands(),
            vec![
                ("/prefixtwo".to_string(), "Test two".to_string()),
                ("/prefix".to_string(), "Test".to_string())
            ]
        );
    }

    // @todo need to add testing infra - HOW?

    //    #[test]
    //    fn test_telegram_retry() {
    //        let stream = stream::iter_result(vec![
    //            Err(failure::Error(telebot::error::ErrorKind::Telegram)),
    //            Ok(19),
    //        ]);
    //        let retry = stream.retry(handle_error).collect()
    //            .then(|x| {
    //                assert_eq!(Ok(vec![19]), x);
    //                Ok(())
    //            });
    //        tokio::run(retry);
    //    }
}
