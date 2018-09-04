use crate::{
    commands::{bot_command::BotCommand, send_html_reply, send_plain_reply},
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
            send_html_reply(
                bot,
                &message,
                "Usage: /psn <b>psnid</b>\nFor example: /psn KPOTA_B_ATEOHE".into(),
            );
            return;
        }

        let name = name.unwrap();

        let from = match message.from {
            None => {
                send_plain_reply(bot, &message, "Message has no sender info.".into());
                return;
            }
            Some(ref from) => from,
        };

        let username = match from.username {
            None => {
                send_plain_reply(
                    bot,
                    &message,
                    "You have no telegram username, register your telegram account first.".into(),
                );
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
                    send_plain_reply(
                        bot,
                        &message,
                        format!(
                            "Your telegram @{username} is already linked with psn {psn}",
                            username = username,
                            psn = user[0].psn_name
                        ),
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

                    send_plain_reply(
                        bot,
                        &message,
                        format!(
                            "Linking telegram @{username} with psn {psn}",
                            username = username,
                            psn = name
                        ),
                    );
                    return;
                }
            }
            Err(_) => {
                send_plain_reply(bot, &message, "Error querying guardian name.".into());
                return;
            }
        };
    }
}
