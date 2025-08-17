use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, ListCommands},
        commands::match_command,
        BotCommand,
    },
    kameo::message::Context,
};

command_actor!(HelpCommand, [ActorUpdateMessage]);

impl BotCommand for HelpCommand {
    fn prefix() -> &'static str {
        "/help"
    }

    fn description() -> &'static str {
        "List available commands"
    }
}

impl Message<ActorUpdateMessage> for HelpCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            let _ = self.bot_ref.tell(ListCommands(message)).await;
        }
    }
}
