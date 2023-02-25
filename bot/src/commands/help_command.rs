use {
    crate::{
        bot_actor::{BotActorMsg, CommandMsg},
        commands::match_command,
        BotCommand,
    },
    ractor::{cast, Actor, ActorProcessingErr},
};

command_actor!(HelpCommand, [ActorUpdateMessage]);

impl BotCommand for HelpCommand {
    fn prefix() -> &'static str {
        "/help"
    }

    fn description() -> &'static str {
        "List available commands"
    }
}

#[async_trait::async_trait]
impl Actor for HelpCommand {
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

    // fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
    async fn handle(
        &self,
        myself: ActorRef<Self>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let (Some(_), _) = match_command(message.text(), Self::prefix(), &self.bot_name) {
            cast!(self.bot_ref, BotActorMsg::ListCommands(message));
        }
        Ok(())
    }
}
