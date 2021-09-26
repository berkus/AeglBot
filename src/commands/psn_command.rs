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
        log::info!("PSN command");

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
        let user_id = from.id;

        let db_user = guardians
            .filter(telegram_id.eq(&user_id))
            .first::<Guardian>(&connection)
            .optional();

        match db_user {
            Ok(Some(user)) => {
                let another_user = guardians
                    .filter(psn_name.ilike(&name))
                    .filter(telegram_id.ne(&user_id))
                    .first::<Guardian>(&connection)
                    .optional();

                match another_user {
                    Ok(Some(_)) => {
                        bot.send_plain_reply(
                            &message,
                            format!(
                                "The psn {psn} is already used by somebody else.",
                                psn = name
                            ),
                        );
                    }
                    Ok(None) => {
                        use diesel_derives_traits::Model;

                        let mut user = user;
                        user.telegram_name = username.to_string();
                        user.psn_name = name.to_string();
                        if user.save(&connection).is_err() {
                            bot.send_plain_reply(
                                &message,
                                "Failed to update telegram and PSN names.".into(),
                            );
                        } else {
                            bot.send_plain_reply(
                                &message,
                                format!(
                                    "Your telegram @{username} is linked with PSN {psn}",
                                    username = username,
                                    psn = name
                                ),
                            );
                        }
                    }
                    Err(_) => {
                        bot.send_plain_reply(&message, "Error querying guardian PSN.".into());
                    }
                }
            }
            Ok(None) => {
                use crate::schema::guardians;

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
                        "Linking telegram @{username} with PSN {psn}",
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
