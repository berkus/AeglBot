use {
    crate::{
        actors::bot_actor::ActorUpdateMessage,
        commands::{guardian_lookup, match_command, validate_username},
        BotCommand,
    },
    kameo::message::Context,
};

command_actor!(WhoisCommand, [ActorUpdateMessage]);

impl WhoisCommand {}

impl BotCommand for WhoisCommand {
    fn prefix() -> &'static str {
        "/whois"
    }

    fn description() -> &'static str {
        "Query telegram or PSN id"
    }
}

impl Message<ActorUpdateMessage> for WhoisCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handle_message(message).await;
    }
}

impl WhoisCommand {
    async fn handle_message(&self, message: ActorUpdateMessage) {
        let connection = self.connection();

        if let (Some(_), name) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if name.is_none() {
                // usage()
                return self
                    .send_reply(
                        &message,
                        "To query user provide his @TelegramId (starting with @) or PsnId",
                    )
                    .await;
            }

            let name = name.unwrap();

            if validate_username(&self.bot_ref, &message, connection)
                .await
                .is_none()
            {
                return; // TODO: say something?
            }

            let guardian = guardian_lookup(&name, connection).await;

            match guardian {
                Ok(Some(guardian)) => {
                    self.send_reply(
                        &message,
                        format!(
                            "Guardian @{telegram_name} PSN {psn_name}",
                            telegram_name = guardian.telegram_name,
                            psn_name = guardian.psn_name
                        ),
                    )
                    .await;
                }
                Ok(None) => {
                    self.send_reply(&message, format!("Guardian {} was not found.", name))
                        .await;
                }
                Err(_) => {
                    self.send_reply(&message, "Error querying guardian name.")
                        .await;
                }
            }
        }
    }
}
