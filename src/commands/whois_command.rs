use crate::{commands::validate_username, commands::guardian_lookup, models::Guardian};
use crate::{Bot, BotCommand, DbConnection};
use diesel::prelude::*;
use futures::Future;

pub struct WhoisCommand;

command_ctor!(WhoisCommand);

impl BotCommand for WhoisCommand {
    fn prefix(&self) -> &'static str {
        "/whois"
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

        let guardian = guardian_lookup(&name, &connection);

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
