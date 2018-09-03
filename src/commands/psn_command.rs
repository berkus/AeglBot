use crate::{
    commands::{bot_command::BotCommand, spawn_message},
    models::{Guardian, NewGuardian},
    schema::guardians::dsl::*,
};
use diesel::{self, pg::PgConnection, prelude::*};
use futures::Future;
use telebot::{functions::*, RcBot};

pub struct PsnCommand;

impl BotCommand for PsnCommand {
    fn prefix() -> &'static str {
        "psn"
    }

    fn description() -> &'static str {
        "Link your telegram user to PSN"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        name: Option<String>,
        connection: &PgConnection,
    ) {
        info!("PSN command");

        if name.is_none() {
            spawn_message(
                bot,
                bot.message(
                    message.chat.id,
                    "Usage: /psn <b>psnid</b>\nFor example: /psn KPOTA_B_ATEOHE".into(),
                ).parse_mode(ParseMode::HTML)
                .reply_to_message_id(message.message_id),
            );
            return;
        }

        let name = name.unwrap();

        let from = match message.from {
            None => {
                spawn_message(
                    bot,
                    bot.message(message.chat.id, "Message has no sender info.".into())
                        .reply_to_message_id(message.message_id),
                );
                return;
            }
            Some(from) => from,
        };

        let username = match from.username {
            None => {
                spawn_message(
                    bot,
                    bot.message(
                        message.chat.id,
                        "You have no telegram username, register your telegram account first."
                            .into(),
                    ).reply_to_message_id(message.message_id),
                );

                return;
            }
            Some(name) => name,
        };

        let db_user = guardians
            .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
            .limit(1)
            .load::<Guardian>(connection);
        match db_user {
            Ok(user) => {
                if user.len() > 0 {
                    spawn_message(
                        bot,
                        bot.message(
                            message.chat.id,
                            format!(
                                "Your telegram @{username} is already linked with psn {psn}",
                                username = username,
                                psn = user[0].psn_name
                            ),
                        ).reply_to_message_id(message.message_id),
                    );
                    return;
                } else {
                    use crate::schema::guardians;

                    let user_id = from.id.into();

                    let guardian = NewGuardian {
                        telegram_name: &username,
                        telegram_id: user_id,
                        psn_name: &name,
                    };

                    diesel::insert_into(guardians::table)
                        .values(&guardian)
                        .execute(connection)
                        .expect("Unexpected error saving guardian");

                    spawn_message(
                        bot,
                        bot.message(
                            message.chat.id,
                            format!(
                                "Linking telegram @{username} with psn {psn}",
                                username = username,
                                psn = name
                            ),
                        ).reply_to_message_id(message.message_id),
                    );
                    return;
                }
            }
            Err(_) => {
                spawn_message(
                    bot,
                    bot.message(message.chat.id, "Error querying guardian name.".into())
                        .reply_to_message_id(message.message_id),
                );
                return;
            }
        };
    }
}
