mod activities_command;
pub use self::activities_command::*;
mod bot_command;
pub use self::bot_command::*;
mod cancel_command;
pub use self::cancel_command::*;
mod details_command;
pub use self::details_command::*;
mod help_command;
pub use self::help_command::*;
mod join_command;
pub use self::join_command::*;
mod lfg_command;
pub use self::lfg_command::*;
mod list_command;
pub use self::list_command::*;
mod psn_command;
pub use self::psn_command::*;
mod whois_command;
pub use self::whois_command::*;

use crate::{models::Guardian, schema::guardians::dsl::*};
use diesel::{pg::PgConnection, prelude::*};
use futures::Future;
use telebot::{functions::*, RcBot};

pub fn decapitalize(s: String) -> String {
    s.chars()
        .nth(0)
        .map(|item| item.to_lowercase().chain(s.chars().skip(1)).collect())
        .unwrap()
}

pub fn spawn_message(bot: &RcBot, m: telebot::functions::WrapperMessage) {
    bot.inner
        .handle
        .spawn(m.send().map(|_| ()).map_err(|e| error!("Error: {:?}", e)));
}

pub fn send_plain_reply(bot: &RcBot, source: &telebot::objects::Message, t: String) {
    spawn_message(
        bot,
        bot.message(source.chat.id, t)
            .reply_to_message_id(source.message_id)
            .disable_notificaton(true)
            .disable_web_page_preview(true),
    );
}

pub fn send_html_reply(bot: &RcBot, source: &telebot::objects::Message, t: String) {
    spawn_message(
        bot,
        bot.message(source.chat.id, t)
            .reply_to_message_id(source.message_id)
            .parse_mode(ParseMode::HTML)
            .disable_notificaton(true)
            .disable_web_page_preview(true),
    );
}

pub fn send_html_message(bot: &RcBot, chat: telebot::objects::Integer, t: String) {
    spawn_message(
        bot,
        bot.message(chat, t)
            .parse_mode(ParseMode::HTML)
            .disable_notificaton(true)
            .disable_web_page_preview(true),
    );
}

pub fn validate_username(
    bot: &RcBot,
    message: &telebot::objects::Message,
    connection: &PgConnection,
) -> Option<Guardian> {
    let username = match message.from.as_ref().unwrap().username {
        None => {
            send_plain_reply(
                bot,
                message,
                "You have no telegram username, register your telegram account first.".into(),
            );
            return None;
        }
        Some(ref name) => name.clone(),
    };

    let db_user = guardians
        .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
        .limit(1)
        .load::<Guardian>(connection);
    match db_user {
        Ok(users) => if users.len() > 0 {
            Some(users[0].clone())
        } else {
            send_plain_reply(
                bot,
                message,
                "You need to link your PSN account first: use /psn command".into(),
            );
            None
        },
        Err(_) => {
            send_plain_reply(bot, message, "Error querying guardian info.".into());
            None
        }
    }
}
