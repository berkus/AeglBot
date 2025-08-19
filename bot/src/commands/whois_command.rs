use {
    crate::{
        BotCommand,
        actors::bot_actor::ActorUpdateMessage,
        commands::{guardian_lookup, match_command, validate_username},
    },
    riker::actors::Tell,
};

command_actor!(WhoisCommand, [ActorUpdateMessage]);

impl WhoisCommand {
    fn send_reply<S>(&self, message: &ActorUpdateMessage, reply: S)
    where
        S: Into<String>,
    {
        self.bot_ref.tell(
            SendMessageReply(reply.into(), message.clone(), Format::Plain, Notify::Off),
            None,
        );
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

impl Receive<ActorUpdateMessage> for WhoisCommand {
    type Msg = WhoisCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), name) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if name.is_none() {
                return self.send_reply(
                    &message,
                    "To query user provide his @TelegramId (starting with @) or PsnId",
                );
            }

            let name = name.unwrap();
            let connection = self.connection();

            if validate_username(&self.bot_ref, &message, &connection).is_none() {
                return; // TODO: say something?
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
                    );
                }
                Ok(None) => {
                    self.send_reply(&message, format!("Guardian {} was not found.", name));
                }
                Err(_) => {
                    self.send_reply(&message, "Error querying guardian name.");
                }
            }
        }
    }
}
