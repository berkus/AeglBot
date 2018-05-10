mod psn_command;
pub use self::psn_command::*;
mod whois_command;
pub use self::whois_command::*;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use models::Guardian;
use schema::guardians::dsl::*;
use telegram_bot;
use telegram_bot::CanReplySendMessage;

pub fn validate_username(
    api: &telegram_bot::Api,
    message: &telegram_bot::Message,
    connection: &PgConnection,
) -> Option<Guardian> {
    let username = match message.from.username {
        None => {
            api.spawn(message.text_reply(
                "You have no telegram username, register your telegram account first.",
            ));
            return None;
        }
        Some(ref name) => name,
    };
    let db_user = guardians
        .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
        .limit(1)
        .load::<Guardian>(connection);
    match db_user {
        Ok(users) => if users.len() > 0 {
            Some(users[0].clone())
        } else {
            api.spawn(
                message.text_reply("You need to link your PSN account first: use /psn command"),
            );
            None
        },
        Err(_) => {
            api.spawn(message.text_reply("Error querying guardian info."));
            None
        }
    }
}
