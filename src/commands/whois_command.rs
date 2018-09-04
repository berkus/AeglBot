use crate::{
    commands::{bot_command::BotCommand, send_plain_reply, validate_username},
    models::Guardian,
    schema::guardians::dsl::*,
};
use diesel::{pg::PgConnection, prelude::*};
use futures::Future;
use telebot::{functions::*, RcBot};

pub struct WhoisCommand;

impl BotCommand for WhoisCommand {
    fn prefix() -> &'static str {
        "whois"
    }

    fn description() -> &'static str {
        "Query telegram or PSN id"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        name: Option<String>,
        connection: &PgConnection,
    ) {
        if name.is_none() {
            send_plain_reply(
                bot,
                message,
                "To query user provide his @TelegramId (starting with @) or PsnId".into(),
            );
            return;
        }

        let name = name.unwrap();

        if let None = validate_username(bot, &message, connection) {
            return;
        }

        let guardian = if name.starts_with("@") {
            guardians
                .filter(telegram_name.eq(&name[1..]))
                .limit(1)
                .load::<Guardian>(connection)
        } else {
            guardians
                .filter(psn_name.eq(&name))
                .limit(1)
                .load::<Guardian>(connection)
        };

        match guardian {
            Ok(guardian) => {
                if guardian.len() > 0 {
                    send_plain_reply(
                        bot,
                        message,
                        format!(
                            "Guardian @{telegram_name} PSN {psn_name}",
                            telegram_name = guardian[0].telegram_name,
                            psn_name = guardian[0].psn_name
                        ),
                    );
                } else {
                    send_plain_reply(bot, message, format!("Guardian {} was not found.", name));
                }
            }
            Err(_) => {
                send_plain_reply(bot, message, "Error querying guardian name.".into());
            }
        }
    }
}
