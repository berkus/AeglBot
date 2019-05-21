use {
    crate::{
        models::{Guardian, NewGuardian},
        schema::guardians::dsl::*,
        Bot, BotCommand, DbConnection,
    },
    diesel::{self, prelude::*},
    futures::Future,
};

pub struct PsnCommand;

command_ctor!(PsnCommand);

impl BotCommand for PsnCommand {
    fn prefix(&self) -> &'static str {
        "/psn"
    }

    fn description(&self) -> &'static str {
        "Link your telegram user to PSN"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        name: Option<String>,
    ) {
        info!("PSN command");

        if name.is_none() {
            return bot.send_html_reply(
                &message,
                "Usage: /psn <b>psnid</b>\nFor example: /psn KPOTA_B_ATEOHE".into(),
            );
        }

        let name = name.unwrap();

        let from = match message.from {
            None => {
                return bot.send_plain_reply(&message, "Message has no sender info.".into());
            }
            Some(ref from) => from,
        };

        let username = match from.username {
            None => {
                return bot.send_plain_reply(
                    &message,
                    "You have no telegram username, register your telegram account first.".into(),
                );
            }
            Some(ref name) => name,
        };

        let connection = bot.connection();

        let db_user = guardians
            .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
            .first::<Guardian>(&connection)
            .optional();

        match db_user {
            Ok(Some(user)) => bot.send_plain_reply(
                &message,
                format!(
                    "Your telegram @{username} is already linked with psn {psn}",
                    username = username,
                    psn = user.psn_name
                ),
            ),
            Ok(None) => {
                use crate::schema::guardians;

                let user_id = from.id;

                let guardian = NewGuardian {
                    telegram_name: &username,
                    telegram_id: user_id,
                    psn_name: &name,
                };

                diesel::insert_into(guardians::table)
                    .values(&guardian)
                    .execute(&connection)
                    .expect("Unexpected error saving guardian");

                bot.send_plain_reply(
                    &message,
                    format!(
                        "Linking telegram @{username} with psn {psn}",
                        username = username,
                        psn = name
                    ),
                );
            }
            Err(_) => {
                bot.send_plain_reply(&message, "Error querying guardian name.".into());
            }
        };
    }
}
