use crate::{
    commands::{extended_command::ExtendedCommand, validate_username},
    models::Guardian,
    schema::guardians::dsl::*,
};
use diesel::{pg::PgConnection, prelude::*};
use telegram_bot::{self, CanReplySendMessage};

pub struct WhoisCommand;

impl ExtendedCommand for WhoisCommand {
    fn prefix() -> &'static str {
        "whois"
    }
    fn description() -> &'static str {
        "Query telegram or PSN id"
    }
}

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

        if let None = validate_username(api, message, connection) {
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
