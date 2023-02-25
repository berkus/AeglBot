#[cfg(target_os = "linux")]
use procfs::process::Process;
use {
    crate::{
        bot_actor::{BotActorMsg, CommandMsg, Format, Notify},
        commands::match_command,
        BotCommand,
    },
    ractor::{cast, Actor, ActorProcessingErr},
};

command_actor!(InfoCommand, [ActorUpdateMessage]);

#[cfg(target_os = "linux")]
fn get_process_info() -> String {
    if let Ok(process) = Process::myself() {
        format!(
            "{thn} threads, {vm} bytes virtual memory, {rm} bytes resident memory",
            thn = process.stat.num_threads,
            vm = process.stat.vsize,
            rm = process.stat.rss_bytes(),
        )
    } else {
        "Couldn't access process information".to_string()
    }
}

#[cfg(not(target_os = "linux"))]
fn get_process_info() -> String {
    "Process info only available on Linux hosts.".to_string()
}

impl BotCommand for InfoCommand {
    fn prefix() -> &'static str {
        "/info"
    }

    fn description() -> &'static str {
        ""
    }
}

#[async_trait::async_trait]
impl Actor for InfoCommand {
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
                    get_process_info(),
                    message,
                    Format::Html,
                    Notify::Off
                )
            );
        }
        Ok(())
    }
}
