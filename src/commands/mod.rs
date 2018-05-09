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
) -> bool {
    let username = match message.from.username {
        None => {
            api.spawn(message.text_reply(
                "You have no telegram username, register your telegram account first.",
            ));
            return false;
        }
        Some(ref name) => name,
    };
    let db_user = guardians
        .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
        .load::<Guardian>(connection);
    let _db_user = match db_user {
        Ok(user) => user,
        Err(_) => {
            api.spawn(
                message.text_reply("You need to link your PSN account first: use /psn command"),
            );
            return false;
        }
    };
    true
}
