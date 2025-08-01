// #![feature(nll)] // features from edition-2018
// #![feature(type_alias_enum_variants)]
// #![allow(proc_macro_derive_resolution_fallback)] // see https://github.com/rust-lang/rust/issues/50504
#![warn(unused_imports)] // during development
#![feature(type_ascription)]
#![expect(non_local_definitions)] // Old diesel macros

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derives_extra;

use {diesel::pg::PgConnection, diesel_logger::LoggingConnection, r2d2::Pool};

pub mod bot_actor;
pub mod commands;
pub mod datetime;
pub mod models;
pub mod schema;
pub mod services;

static TEMPLATE_FILES: std::sync::LazyLock<include_dir::Dir<'_>> =
    std::sync::LazyLock::new(|| include_dir::include_dir!("$CARGO_MANIFEST_DIR/templates"));

pub(crate) static TEMPLATES: std::sync::LazyLock<tera::Tera> = std::sync::LazyLock::new(|| {
    let mut tera = tera::Tera::default();
    for file in TEMPLATE_FILES.find("**/*.tera").unwrap() {
        if let Some(template) = file.as_file() {
            tera.add_raw_template(
                template.path().with_extension("").to_str().unwrap(), // drop .tera extension
                template.contents_utf8().unwrap(),
            )
            .unwrap();
        }
    }
    tera
});

#[allow(
    clippy::crate_in_macro_def,
    reason = "We refer to this specific TEMPLATES instance in this specific crate"
)]
#[macro_export]
macro_rules! render_template {
    ($template:expr) => {
        {
            crate::TEMPLATES.render($template, &tera::Context::new())
                .map_err(|e| format!("Failed to render template '{}': {}", $template, e))
        }
    };
    ($template:expr, $(($key:expr,$value:expr)),+) => {
        {
            let mut context = tera::Context::new();
            $(
                context.insert($key, $value);
            )*
            crate::TEMPLATES.render($template, &context)
                .map_err(|e| format!("Failed to render template '{}': {}", $template, e))
        }
    };
}

// TODO: only BotConnection should be public
pub type DbConnection = LoggingConnection<PgConnection>;
pub type DbConnPool = Pool<diesel::r2d2::ConnectionManager<DbConnection>>;
pub type BotConnection = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<DbConnection>>;

pub trait NamedActor {
    fn actor_name() -> String;
}

pub trait BotCommand {
    /// Print command usage instructions.
    // fn usage(&self, bot: &BotMenu, message: &UpdateWithCx<Bot>, Message>);
    /// Return command prefix to match.
    /// To support sub-commands the prefix for root commands should start with '/'.
    fn prefix() -> &'static str;
    /// Return command description.
    fn description() -> &'static str;
}

/// Establish a pool of connections with DB.
pub fn establish_db_connection() -> DbConnPool {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = diesel::r2d2::ConnectionManager::new(database_url.clone());

    r2d2::Pool::builder()
        .min_idle(Some(1))
        .max_size(15)
        .build(manager)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
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
    // use super::*;

    // Command is prefix of another command.
    // struct PrefixCommand;

    // struct PrefixTwoCommand;

    // impl PrefixCommand {
    //     pub fn new() -> Box<Self> {
    //         Box::new(PrefixCommand)
    //     }
    // }

    // impl BotCommand for PrefixCommand {
    //     fn prefix() -> &'static str {
    //         "/prefix"
    //     }

    //     fn description() -> &'static str {
    //         "Test"
    //     }
    // }

    // impl PrefixTwoCommand {
    //     pub fn new() -> Box<Self> {
    //         Box::new(PrefixTwoCommand)
    //     }
    // }

    // impl BotCommand for PrefixTwoCommand {
    //     fn prefix() -> &'static str {
    //         "/prefixtwo"
    //     }

    //     fn description() -> &'static str {
    //         "Test two"
    //     }
    // }

    // #[test]
    // fn test_command_insertion_order1() {
    //     dotenv().ok();
    //     let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
    //     let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    //     let mut bot = Bot::new(&bot_name, &token);
    //
    //     bot.register_command(PrefixCommand::new());
    //     bot.register_command(PrefixTwoCommand::new());
    //
    //     assert_eq!(
    //         bot.list_commands(),
    //         vec![
    //             ("/prefixtwo".to_string(), "Test two".to_string()),
    //             ("/prefix".to_string(), "Test".to_string())
    //         ]
    //     );
    // }

    // #[test]
    // fn test_command_insertion_order2() {
    //     dotenv().ok();
    //     let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
    //     let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    //     let mut bot = Bot::new(&bot_name, &token);
    //
    //     bot.register_command(PrefixTwoCommand::new());
    //     bot.register_command(PrefixCommand::new());
    //
    //     assert_eq!(
    //         bot.list_commands(),
    //         vec![
    //             ("/prefixtwo".to_string(), "Test two".to_string()),
    //             ("/prefix".to_string(), "Test".to_string())
    //         ]
    //     );
    // }

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
