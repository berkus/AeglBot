use {
    crate::{
        bot_actor::{ActorUpdateMessage, BotActorMsg, Format, Notify},
        commands::match_command,
        services::this_week_in_d2,
        BotCommand,
    },
    ractor::{cast, Actor, ActorProcessingErr},
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

#[async_trait::async_trait]
impl Actor for D2weekCommand {
    type Msg = ActorUpdateMessage;
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
                    this_week_in_d2(),
                    message,
                    Format::Markdown,
                    Notify::Off
                )
            );
        }
        Ok(())
    }
}
