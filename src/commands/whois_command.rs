use crate::{commands::validate_username, models::Guardian, schema::guardians::dsl::*};
use crate::{Bot, BotCommand, DbConnection};
use diesel::prelude::*;
use futures::Future;

pub struct WhoisCommand;

impl BotCommand for WhoisCommand {
    fn prefix(&self) -> &'static str {
        "whois"
    }

    fn description(&self) -> &'static str {
        "Query telegram or PSN id"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        name: Option<String>,
    ) {
        if name.is_none() {
            return bot.send_plain_reply(
                &message,
                "To query user provide his @TelegramId (starting with @) or PsnId".into(),
            );
        }

        let name = name.unwrap();
        let connection = bot.connection();

        if validate_username(bot, &message, &connection).is_none() {
            return;
        }

        let guardian = if name.starts_with('@') {
            guardians
                .filter(telegram_name.eq(&name[1..]))
                .first::<Guardian>(&connection)
                .optional()
        } else {
            guardians
                .filter(psn_name.eq(&name))
                .first::<Guardian>(&connection)
                .optional()
        };

        match guardian {
            Ok(Some(guardian)) => {
                bot.send_plain_reply(
                    &message,
                    format!(
                        "Guardian @{telegram_name} PSN {psn_name}",
                        telegram_name = guardian.telegram_name,
                        psn_name = guardian.psn_name
                    ),
                );
            }
            Ok(None) => {
                bot.send_plain_reply(&message, format!("Guardian {} was not found.", name));
            }
            Err(_) => {
                bot.send_plain_reply(&message, "Error querying guardian name.".into());
            }
        }
    }
}
