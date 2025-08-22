use crate::{actors::bot_actor::ActorUpdateMessage, commands::match_command};

command_actor!(ChatIdCommand, "chatid", "Figure out the numeric chat ID");

impl Message<ActorUpdateMessage> for ChatIdCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), _) = match_command(msg.update.text(), Self::prefix(), &self.bot_name) {
            self.send_reply(&msg, format!("✅ Current chat id: {}", msg.update.chat.id))
                .await;
        }
    }
}
