use {
    crate::{
        BotCommand,
        actors::bot_actor::{ActorUpdateMessage, ListCommands},
        commands::match_command,
    },
    riker::actors::Tell,
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

impl Receive<ActorUpdateMessage> for HelpCommand {
    type Msg = HelpCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            self.bot_ref.tell(ListCommands(message), None);
        }
    }
}
