use crate::{Bot, BotCommand, DbConnection};
#[cfg(linux)]
use procfs::Process;

pub struct InfoCommand;

impl InfoCommand {
    pub fn new() -> Box<Self> {
        Box::new(InfoCommand)
    }
}

#[cfg(linux)]
fn get_process_info() -> String {
    match Process::myself() {
        Ok(process) => format!(
            "{thn} threads, {vm} bytes virtual memory, {rm} bytes resident memory",
            process.stat.num_threads,
            process.stat.vsize,
            process.stat.rss_bytes(),
        ),
        Err(_) => "Couldn't access process information".to_string(),
    }
}

#[cfg(not(linux))]
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
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _name: Option<String>,
    ) {
        bot.send_html_reply(&message, get_process_info());
    }
}
