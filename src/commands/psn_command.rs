use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use models::{Guardian, NewGuardian};
use schema::guardians::dsl::*;
use telegram_bot::{self, CanReplySendMessage, Integer};

pub struct PsnCommand; //ExtendedCommand("psn", "Link your telegram user to PSN")

impl PsnCommand {
    pub fn handle(
        api: &telegram_bot::Api,
        message: &telegram_bot::Message,
        name: &String,
        connection: &PgConnection,
    ) {
        if name.len() < 1 {
            api.spawn(
                message.text_reply("Usage: /psn <b>psnid</b>\nFor example: /psn KPOTA_B_ATEOHE"),
            );
            return;
        }

        let username = match message.from.username {
            None => {
                api.spawn(message.text_reply(
                    "You have no telegram username, register your telegram account first.",
                ));
                return;
            }
            Some(ref name) => name,
        };

        let db_user = guardians
            .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
            .limit(1)
            .load::<Guardian>(connection);
        match db_user {
            Ok(user) => {
                if user.len() > 0 {
                    api.spawn(message.text_reply(format!(
                        "Your telegram @{username} is already linked with psn {psn}",
                        username = username,
                        psn = user[0].psn_name
                    )));
                } else {
                    use schema::guardians;

                    let user_id: Integer = message.from.id.into();

                    let guardian = NewGuardian {
                        telegram_name: username,
                        telegram_id: user_id,
                        psn_name: name,
                    };

                    diesel::insert_into(guardians::table)
                        .values(&guardian)
                        .execute(connection)
                        .expect("Unexpected error saving guardian");

                    api.spawn(message.text_reply(format!(
                        "Linking telegram @{username} with psn {psn}",
                        username = username,
                        psn = name
                    )));
                }
            }
            Err(_) => {
                api.spawn(message.text_reply("Error querying guardian name."));
            }
        };
    }
}
