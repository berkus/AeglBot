use crate::{bot_actor::ActorUpdateMessage, commands::match_command, BotCommand};

command_actor!(ChatidCommand, [ActorUpdateMessage]);

impl BotCommand for ChatidCommand {
    fn prefix() -> &'static str {
        "/chatid"
    }

    fn description() -> &'static str {
        "Figure out the numeric chat ID"
    }
}

impl Message<ActorUpdateMessage> for ChatidCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), _) = match_command(msg.update.text(), Self::prefix(), &self.bot_name) {
            let _ = self
                .send_reply(&msg, format!("ChatId: {}", msg.update.chat.id))
                .await;
        }
    }
}
