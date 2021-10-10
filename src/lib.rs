#![feature(nll)] // features from edition-2018
// #![feature(type_alias_enum_variants)]
#![allow(proc_macro_derive_resolution_fallback)] // see https://github.com/rust-lang/rust/issues/50504
#![allow(unused_imports)] // during development
#![feature(type_ascription)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel_derives_extra;

use {
    diesel::{pg::PgConnection, prelude::*},
    diesel_logger::LoggingConnection,
    r2d2::Pool,
};

pub mod bot_actor;
pub mod commands;
pub mod datetime;
pub mod models;
pub mod schema;
pub mod services;

// TODO: only BotConnection should be public
pub type DbConnection = LoggingConnection<PgConnection>;
pub type DbConnPool = Pool<diesel::r2d2::ConnectionManager<DbConnection>>;
pub type BotConnection = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<DbConnection>>;

pub trait NamedActor {
    fn actor_name() -> String;
}

pub trait BotCommand {
    /// Print command usage instructions.
    // fn usage(&self, bot: &BotMenu, message: &UpdateWithCx<AutoSend<Bot>, Message>);
    /// Return command prefix to match.
    /// To support sub-commands the prefix for root commands should start with '/'.
    fn prefix(&self) -> &'static str;
    /// Return command description.
    fn description(&self) -> &'static str;
    // Execute matched command.
    // fn execute(
    //     &self,
    //     bot: &BotMenu,
    //     message: &UpdateMessage,
    //     command: Option<String>,
    //     text: Option<String>,
    // );
}

// https://chaoslibrary.blot.im/rust-cloning-a-trait-object/
//
// trait BotCommandClone {
//     fn clone_box(&self) -> Box<dyn BotCommand>;
// }
//
// impl<T> BotCommandClone for T
// where
//     T: 'static + BotCommand + Clone,
// {
//     fn clone_box(&self) -> Box<dyn BotCommand> {
//         Box::new(self.clone())
//     }
// }
//
// impl Clone for Box<dyn BotCommand> {
//     fn clone(&self) -> Box<dyn BotCommand> {
//         self.clone_box()
//     }
// }

#[cfg(test)]
mod tests {
    use {super::*, tokio_core::reactor::Core};

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
            _message: &UpdateMessage,
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
        let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
        let mut bot = Bot::new(&bot_name, &token);

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
        let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
        let mut bot = Bot::new(&bot_name, &token);

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
