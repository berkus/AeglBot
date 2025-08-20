use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format},
        commands::match_command,
    },
    kameo::message::Context,
    libbot::services::destiny_schedule::this_week_in_d2,
};

command_actor!(D2weekCommand, "d2week", "Show current Destiny 2 week");

impl Message<ActorUpdateMessage> for D2weekCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            self.send_reply_with_format(&message, this_week_in_d2(), Format::Markdown)
                .await;
        }
    }
}
