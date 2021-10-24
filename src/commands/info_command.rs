#[cfg(target_os = "linux")]
use procfs::process::Process;
use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::match_command,
        BotCommand,
    },
    riker::actors::Tell,
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
        "Show bot info"
    }
}

impl Receive<ActorUpdateMessage> for InfoCommand {
    type Msg = InfoCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), _) = match_command(msg.update.text(), Self::prefix(), &self.bot_name) {
            self.bot_ref.tell(
                SendMessageReply(get_process_info(), msg, Format::Html, Notify::Off),
                None,
            );
        }
    }
}
