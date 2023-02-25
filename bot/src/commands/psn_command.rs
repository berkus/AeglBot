use {
    crate::{
        bot_actor::{BotActorMsg, CommandMsg, Format, Notify},
        commands::match_command,
        models::{Guardian, NewGuardian},
        schema::guardians::dsl::*,
        BotCommand,
    },
    diesel::{self, prelude::*},
    ractor::{cast, Actor, ActorProcessingErr},
    teloxide::prelude::UserId,
};

command_actor!(PsnCommand, [ActorUpdateMessage]);

impl PsnCommand {
    fn send_reply<S>(
        &self,
        message: &CommandMsg,
        reply: S,
        format: Format,
    ) -> Result<(), ActorProcessingErr>
    where
        S: Into<String>,
    {
        cast!(
            self.bot_ref,
            BotActorMsg::SendMessageReply(reply.into(), message.clone(), format, Notify::Off)
        );
        Ok(())
    }
}

impl BotCommand for PsnCommand {
    fn prefix() -> &'static str {
        "/psn"
    }

    fn description() -> &'static str {
        ""
    }
}

#[async_trait::async_trait]
impl Actor for PsnCommand {
    type Msg = CommandMsg;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        todo!()
    }

    // fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
    async fn handle(
        &self,
        myself: ActorRef<Self>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let (Some(_), name) = match_command(message.text(), Self::prefix(), &self.bot_name) {
            log::info!("PSN command");

            if name.is_none() {
                return self.send_reply(
                    &message,
                    "Usage: /psn <b>psnid</b>\nFor example: /psn KPOTA_B_ATEOHE",
                    Format::Html,
                );
            }

            let name = name.unwrap();

            let from = match message.from() {
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
            let UserId(user_id) = from.id;
            let user_id: i64 = user_id.try_into()?;

            let db_user = guardians
                .filter(telegram_id.eq(user_id))
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
                            )?;
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
                                )?;
                            } else {
                                self.send_reply(
                                    &message,
                                    format!(
                                        "Your telegram @{username} is linked with PSN {psn}",
                                        username = username,
                                        psn = name
                                    ),
                                    Format::Plain,
                                )?;
                            }
                        }
                        Err(_) => {
                            self.send_reply(
                                &message,
                                "Error querying guardian PSN.",
                                Format::Plain,
                            )?;
                        }
                    }
                }
                Ok(None) => {
                    use crate::schema::guardians;

                    let guardian = NewGuardian {
                        telegram_name: username,
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
                    )?;
                }
                Err(_) => {
                    self.send_reply(&message, "Error querying guardian name.", Format::Plain)?;
                }
            };
        }
        Ok(())
    }
}
