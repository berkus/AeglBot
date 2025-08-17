use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format},
        commands::match_command,
        BotCommand,
    },
    kameo::message::Context,
    libbot::services::destiny_schedule::this_week_in_d1,
};

command_actor!(D1weekCommand, [ActorUpdateMessage]);

impl BotCommand for D1weekCommand {
    fn prefix() -> &'static str {
        "/dweek"
    }

    fn description() -> &'static str {
        "Show current Destiny 1 week"
    }
}

impl Message<ActorUpdateMessage> for D1weekCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            let _ = self
                .send_reply_with_format(&message, this_week_in_d1(), Format::Markdown)
                .await;
        }
    }
}
