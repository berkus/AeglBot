use {
    crate::{
        bot_actor::{BotActorMsg, CommandMsg, Format, Notify},
        commands::match_command,
        BotCommand,
    },
    ractor::{cast, Actor, ActorProcessingErr},
};

command_actor!(ChatidCommand, [ActorUpdateMessage]);

impl BotCommand for ChatidCommand {
    fn prefix() -> &'static str {
        "/chatid"
    }

    fn description() -> &'static str {
        ""
    }
}

#[async_trait::async_trait]
impl Actor for ChatidCommand {
    type Msg = CommandMsg;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        todo!()
    }

    // fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ActorUpdateMessage, _sender: Sender) {
    async fn handle(
        &self,
        myself: ActorRef<Self>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let (Some(_), _) = match_command(message.text(), Self::prefix(), &self.bot_name) {
            cast!(
                self.bot_ref,
                BotActorMsg::SendMessageReply(
                    format!("ChatId: {}", message.chat.id),
                    message,
                    Format::Plain,
                    Notify::Off,
                )
            );
        }
        Ok(())
    }
}
