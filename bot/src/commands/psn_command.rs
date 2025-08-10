use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::match_command,
        BotCommand,
    },
    entity::guardians,
    riker::actors::Tell,
    sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set},
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

    fn usage(&self, message: &ActorUpdateMessage) {
        self.send_reply(
            message,
            "Usage: /psn <b>psnid</b>\nFor example: /psn KPOTA_B_ATEOHE",
            Format::Html,
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
        tokio::runtime::Handle::current().block_on(async {
            self.handle_message(message).await;
        });
    }
}

impl PsnCommand {
    async fn handle_message(&self, message: ActorUpdateMessage) {
        let connection = self.connection();

        if let (Some(_), name) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            log::info!("PSN command");

            if name.is_none() {
                return self.usage(&message);
            }

            let name = name.unwrap();

            let from = match &message.update.from {
                None => {
                    return self.send_reply(&message, "Message has no sender info.", Format::Plain);
                }
                Some(from) => from,
            };

            let username = match &from.username {
                None => {
                    return self.send_reply(
                        &message,
                        "You have no telegram username, register your telegram account first.",
                        Format::Plain,
                    );
                }
                Some(name) => name,
            };

            let user_id = from.id;

            let db_user = guardians::Entity::find()
                .filter(guardians::Column::TelegramId.eq(user_id.0 as i64))
                .one(connection)
                .await;

            match db_user {
                Ok(Some(user)) => {
                    let another_user = guardians::Entity::find()
                        .filter(guardians::Column::PsnName.contains(&name))
                        .filter(guardians::Column::TelegramId.ne(user_id.0 as i64))
                        .one(connection)
                        .await;

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
                            let mut user: guardians::ActiveModel = user.into();
                            user.telegram_name = Set(username.to_string());
                            user.psn_name = Set(name.to_string());

                            if user.update(connection).await.is_err() {
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
                    let guardian = guardians::ActiveModel {
                        telegram_name: Set(username.to_string()),
                        telegram_id: Set(user_id.0 as i64),
                        psn_name: Set(name.to_string()),
                        ..Default::default()
                    };

                    if guardian.insert(connection).await.is_err() {
                        self.send_reply(
                            &message,
                            "Unexpected error saving guardian",
                            Format::Plain,
                        );
                    } else {
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
                }
                Err(_) => {
                    self.send_reply(&message, "Error querying guardian name.", Format::Plain);
                }
            };
        }
    }
}
