use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::match_command,
        BotCommand,
    },
    riker::actors::Tell,
};

command_actor!(ChatidCommand, [ActorUpdateMessage]);

impl BotCommand for ChatidCommand {
    fn prefix() -> &'static str {
        "/chatid"
    }

    fn description() -> &'static str {
        "Figure out the numeric chat ID"
    }
}

impl Receive<ActorUpdateMessage> for ChatidCommand {
    type Msg = ChatidCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), _) = match_command(msg.update.text(), Self::prefix(), &self.bot_name) {
            self.bot_ref.tell(
                SendMessageReply(
                    format!("ChatId: {}", msg.update.chat.id),
                    msg,
                    Format::Plain,
                    Notify::Off,
                ),
                None,
            );
        }
    }
}
