use commands::validate_username;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use models::Guardian;
use schema::guardians::dsl::*;
use telegram_bot;
use telegram_bot::CanReplySendMessage;

pub struct WhoisCommand; //ExtendedCommand("whois", "Query telegram or PSN id")

impl WhoisCommand {
    pub fn handle(
        api: &telegram_bot::Api,
        message: &telegram_bot::Message,
        name: &String,
        connection: &PgConnection,
    ) {
        if name.len() < 1 {
            api.spawn(
                message
                    .text_reply("To query user provide his @TelegramId (starting with @) or PsnId"),
            );
            return;
        }

        if !validate_username(api, message, connection) {
            return;
        };

        let guardian = if name.starts_with("@") {
            guardians
                .filter(telegram_name.eq(&name[1..]))
                .limit(1)
                .load::<Guardian>(connection)
        } else {
            guardians
                .filter(psn_name.eq(name))
                .limit(1)
                .load::<Guardian>(connection)
        };

        match guardian {
            Ok(guardian) => {
                if guardian.len() > 0 {
                    api.spawn(message.text_reply(format!(
                        "Guardian @{telegram_name} PSN {psn_name}",
                        telegram_name = guardian[0].telegram_name,
                        psn_name = guardian[0].psn_name
                    )));
                } else {
                    api.spawn(
                        message.text_reply(format!("Guardian {name} was not found.", name = name)),
                    );
                }
            }
            Err(_) => {
                api.spawn(message.text_reply("Error querying guardian name."));
            }
        }
    }
}
