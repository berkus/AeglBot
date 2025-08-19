use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::match_command,
        BotCommand,
    },
    libbot::services::destiny_schedule::this_week_in_d1,
    riker::actors::Tell,
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

impl Receive<ActorUpdateMessage> for D1weekCommand {
    type Msg = D1weekCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), _) = match_command(msg.update.text(), Self::prefix(), &self.bot_name) {
            self.bot_ref.tell(SendMessageReply(
                this_week_in_d1(),
                msg,
                Format::Markdown,
                Notify::Off,
            ));
        }
    }
}
