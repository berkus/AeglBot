use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::match_command,
        BotCommand,
    },
    libbot::services::destiny_schedule::this_week_in_d2,
    riker::actors::Tell,
};

command_actor!(D2weekCommand, [ActorUpdateMessage]);

impl BotCommand for D2weekCommand {
    fn prefix() -> &'static str {
        "/d2week"
    }

    fn description() -> &'static str {
        "Show current Destiny 2 week"
    }
}

impl Receive<ActorUpdateMessage> for D2weekCommand {
    type Msg = D2weekCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), _) = match_command(msg.update.text(), Self::prefix(), &self.bot_name) {
            self.bot_ref.tell(
                SendMessageReply(this_week_in_d2(), msg, Format::Markdown, Notify::Off),
                None,
            );
        }
    }
}
