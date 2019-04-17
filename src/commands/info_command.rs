#[cfg(target_os = "linux")]
use procfs::{ProcResult, Process};
use {
    crate::{BotCommand, BotMenu, DbConnection},
    teloxide::prelude::*,
};

pub struct InfoCommand;

command_ctor!(InfoCommand);

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
    fn prefix(&self) -> &'static str {
        "/info"
    }

    fn description(&self) -> &'static str {
        "Show bot info"
    }

    fn execute(
        &self,
        bot: &BotMenu,
        message: &UpdateWithCx<AutoSend<Bot>, Message>,
        _command: Option<String>,
        _name: Option<String>,
    ) {
        bot.send_html_reply(&message, get_process_info());
    }
}
