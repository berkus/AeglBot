use {
    crate::{actors::bot_actor::ActorUpdateMessage, commands::match_command},
    entity::guardians,
    kameo::message::Context,
    sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set},
};

command_actor!(PsnCommand, "psn", "Link your telegram user to PSN");

impl Message<ActorUpdateMessage> for PsnCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), name) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            log::info!("PSN command");

            if name.is_none() {
                return self.usage(&message).await;
            }

            let name = name.unwrap();

            let from = match &message.update.from {
                None => {
                    return self
                        .send_reply(&message, "Message has no sender info.")
                        .await;
                }
                Some(from) => from,
            };

            let username = match &from.username {
                None => {
                    return self.usage(&message).await;
                }
                Some(name) => name,
            };

            let user_id = from.id;

            let connection = self.connection();

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
                                    "❌ The psn {psn} is already used by somebody else.",
                                    psn = name
                                ),
                            )
                            .await;
                        }
                        Ok(None) => {
                            let mut user: guardians::ActiveModel = user.into();
                            user.telegram_name = Set(username.to_string());
                            user.psn_name = Set(name.to_string());

                            if user.update(connection).await.is_err() {
                                self.send_reply(&message, "❌ Failed to update PSN name.")
                                    .await;
                            } else {
                                self.send_reply(
                                    &message,
                                    format!("✅ Your PSN name updated to {}.", name),
                                )
                                .await;
                            }
                        }
                        Err(_) => {
                            self.send_reply(&message, "❌ Error querying guardian PSN.")
                                .await;
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
                        self.send_reply(&message, "❌ Error saving guardian info.")
                            .await;
                    } else {
                        self.send_reply(
                            &message,
                            format!("✅ Your PSN name is now set to {name}."),
                        )
                        .await;
                    }
                }
                Err(_) => {
                    self.send_reply(&message, "❌ Error querying guardian name.")
                        .await;
                }
            };
        }
    }
}
