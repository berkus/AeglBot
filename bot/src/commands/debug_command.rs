use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Debug},
        commands::match_command,
    },
    culpa::throws,
    kameo::message::Context,
};

command_actor!(DebugCommand, "debug", "Run debug operations");

impl Message<ActorUpdateMessage> for DebugCommand {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(&mut self, msg: ActorUpdateMessage, _ctx: &mut Context<Self, Self::Reply>) {
        if let (Some(_), _) = match_command(msg.update.text(), Self::prefix(), &self.bot_name) {
            self.send_reply(&msg, format!("Alive: {}", self.bot_ref.ask(Debug).await?))
                .await;
        }
    }
}
