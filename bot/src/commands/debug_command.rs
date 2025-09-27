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
        if let (Some(_), Some(t)) = match_command(msg.update.text(), Self::prefix(), &self.bot_name)
        {
            let r = libbot::datetime::parse_time_spec(t);
            self.send_reply(&msg, format!("Date: {r:?}")).await;
        }
    }
}
