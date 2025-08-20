use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, ListCommands},
        commands::match_command,
    },
    culpa::throws,
    kameo::message::Context,
};

command_actor!(HelpCommand, "help", "List available commands");

impl Message<ActorUpdateMessage> for HelpCommand {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(&mut self, message: ActorUpdateMessage, _ctx: &mut Context<Self, Self::Reply>) {
        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            self.bot_ref.tell(ListCommands(message)).await?;
        }
    }
}
