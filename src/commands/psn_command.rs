use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::match_command,
        models::{Guardian, NewGuardian},
        schema::guardians::dsl::*,
        BotCommand,
    },
    diesel::{self, prelude::*},
    riker::actors::Tell,
};

command_actor!(PsnCommand, [ActorUpdateMessage]);

impl PsnCommand {
    fn send_reply<S>(&self, message: &ActorUpdateMessage, reply: S, format: Format)
    where
        S: Into<String>,
    {
        self.bot_ref.tell(
            SendMessageReply(reply.into(), message.clone(), format, Notify::Off),
            None,
        );
    }
}

impl BotCommand for PsnCommand {
    fn prefix() -> &'static str {
        "/psn"
    }

    fn description() -> &'static str {
        "Link your telegram user to PSN"
    }
}

impl Receive<ActorUpdateMessage> for PsnCommand {
    type Msg = PsnCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), name) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            log::info!("PSN command");

            if name.is_none() {
                return self.send_reply(
                    &message,
                    "Usage: /psn <b>psnid</b>\nFor example: /psn KPOTA_B_ATEOHE",
                    Format::Html,
                );
            }

            let name = name.unwrap();

            let from = match message.update.from() {
                None => {
                    return self.send_reply(&message, "Message has no sender info.", Format::Plain);
                }
                Some(from) => from,
            };

            let username = match from.username {
                None => {
                    return self.send_reply(
                        &message,
                        "You have no telegram username, register your telegram account first.",
                        Format::Plain,
                    );
                }
                Some(ref name) => name,
            };

            let connection = self.connection();
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
                            self.send_reply(
                                &message,
                                format!(
                                    "The psn {psn} is already used by somebody else.",
                                    psn = name
                                ),
                                Format::Plain,
                            );
                        }
                        Ok(None) => {
                            use diesel_derives_traits::Model;

                            let mut user = user;
                            user.telegram_name = username.to_string();
                            user.psn_name = name.to_string();
                            if user.save(&connection).is_err() {
                                self.send_reply(
                                    &message,
                                    "Failed to update telegram and PSN names.",
                                    Format::Plain,
                                );
                            } else {
                                self.send_reply(
                                    &message,
                                    format!(
                                        "Your telegram @{username} is linked with PSN {psn}",
                                        username = username,
                                        psn = name
                                    ),
                                    Format::Plain,
                                );
                            }
                        }
                        Err(_) => {
                            self.send_reply(
                                &message,
                                "Error querying guardian PSN.",
                                Format::Plain,
                            );
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

                    self.send_reply(
                        &message,
                        format!(
                            "Linking telegram @{username} with PSN {psn}",
                            username = username,
                            psn = name
                        ),
                        Format::Plain,
                    );
                }
                Err(_) => {
                    self.send_reply(&message, "Error querying guardian name.", Format::Plain);
                }
            };
        }
    }
}
