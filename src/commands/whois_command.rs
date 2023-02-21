use {
    crate::{
        bot_actor::{ActorUpdateMessage, BotActorMsg, Format, Notify},
        commands::{guardian_lookup, match_command, validate_username},
        BotCommand,
    },
    ractor::{cast, Actor, ActorProcessingErr},
};

command_actor!(WhoisCommand, [ActorUpdateMessage]);

impl WhoisCommand {
    fn send_reply<S>(
        &self,
        message: &ActorUpdateMessage,
        reply: S,
    ) -> Result<(), ActorProcessingErr>
    where
        S: Into<String>,
    {
        cast!(
            self.bot_ref,
            BotActorMsg::SendMessageReply(
                reply.into(),
                message.clone(),
                Format::Plain,
                Notify::Off
            )
        );
        Ok(())
    }
}

impl BotCommand for WhoisCommand {
    fn prefix() -> &'static str {
        "/whois"
    }

    fn description() -> &'static str {
        "Query telegram or PSN id"
    }
}

#[async_trait::async_trait]
impl Actor for WhoisCommand {
    type Msg = ActorUpdateMessage;
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
            if name.is_none() {
                return self.send_reply(
                    &message,
                    "To query user provide his @TelegramId (starting with @) or PsnId",
                );
            }

            let name = name.unwrap();
            let connection = self.connection();

            if validate_username(&self.bot_ref, &message, &connection).is_none() {
                return Ok(()); // TODO: say something?
            }

            let guardian = guardian_lookup(&name, &connection);

            match guardian {
                Ok(Some(guardian)) => {
                    self.send_reply(
                        &message,
                        format!(
                            "Guardian @{telegram_name} PSN {psn_name}",
                            telegram_name = guardian.telegram_name,
                            psn_name = guardian.psn_name
                        ),
                    )?;
                }
                Ok(None) => {
                    self.send_reply(&message, format!("Guardian {} was not found.", name));
                }
                Err(_) => {
                    self.send_reply(&message, "Error querying guardian name.");
                }
            }
        }
        Ok(())
    }
}
