#[macro_export]
macro_rules! command_ctor {
    ($name:ident) => {
        impl $name {
            pub fn new() -> Box<Self> {
                Box::new($name)
            }
        }
    };
}

mod activities_command;
pub use self::activities_command::*;
mod cancel_command;
pub use self::cancel_command::*;
mod d2week_command;
pub use self::d2week_command::*;
mod dweek_command;
pub use self::dweek_command::*;
mod edit_command;
pub use self::edit_command::*;
mod editguar_command;
pub use self::editguar_command::*;
mod help_command;
pub use self::help_command::*;
mod info_command;
pub use self::info_command::*;
mod join_command;
pub use self::join_command::*;
mod lfg_command;
pub use self::lfg_command::*;
mod list_command;
pub use self::list_command::*;
mod manage_command;
pub use self::manage_command::*;
mod psn_command;
pub use self::psn_command::*;
mod whois_command;
pub use self::whois_command::*;

use crate::{models::Guardian, schema::guardians::dsl::*};
use crate::{Bot, DbConnection};
use diesel::prelude::*;
use futures::Future;
//use failure::Error;

pub fn decapitalize(s: &str) -> String {
    s.chars()
        .nth(0)
        .map(|item| item.to_lowercase().chain(s.chars().skip(1)).collect())
        .unwrap()
}

/// Return a guardian record if message author is registered in Guardians table, `None` otherwise.
pub fn validate_username(
    bot: &Bot,
    message: &telebot::objects::Message,
    connection: &DbConnection,
) -> Option<Guardian> {
    let username = match message.from.as_ref().unwrap().username {
        None => {
            bot.send_plain_reply(
                message,
                "You have no telegram username, register your telegram account first.".into(),
            );
            return None;
        }
        Some(ref name) => name.clone(),
    };

    let db_user = guardians
        .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
        .first::<Guardian>(connection)
        .optional();

    match db_user {
        Ok(Some(user)) => Some(user),
        Ok(None) => {
            bot.send_plain_reply(
                message,
                "You need to link your PSN account first: use /psn command".into(),
            );
            None
        }
        Err(_) => {
            bot.send_plain_reply(message, "Error querying guardian info.".into());
            None
        }
    }
}

/// Return a guardian record if message author is an admin user, `None` otherwise.
pub fn admin_check(
    bot: &Bot,
    message: &telebot::objects::Message,
    connection: &DbConnection,
) -> Option<Guardian> {
    validate_username(bot, message, connection).filter(|g| g.is_admin)
}

pub fn guardian_lookup(
    name: &str,
    connection: &DbConnection,
) -> Result<Option<Guardian>, diesel::result::Error> {
    if name.starts_with('@') {
        guardians
            .filter(telegram_name.eq(&name[1..]))
            .first::<Guardian>(connection)
            .optional()
    } else {
        guardians
            .filter(psn_name.ilike(&name))
            .first::<Guardian>(connection)
            .optional()
    }
    // @todo: lookup by integer id, positive
}
