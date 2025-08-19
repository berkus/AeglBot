#[cfg(target_os = "linux")]
use procfs::process::Process;
use {
    crate::{bot_actor::ActorUpdateMessage, commands::match_command, BotCommand},
    kameo::message::Context,
};

command_actor!(UptimeCommand, [ActorUpdateMessage]);

#[cfg(target_os = "linux")]
fn get_process_info() -> String {
    if let Ok(process) = Process::myself() {
        use thousands::Separable;
        let stat = process.stat().unwrap();
        let page_size = procfs::page_size();
        format!(
            "- 🧵 {thn} threads\n- 📃 {vmb} bytes ({vmp} pages) virtual memory\n- 📃 {rmb} bytes ({rmp} pages) resident memory",
            thn = stat.num_threads,
            vmb = stat.vsize.separate_with_commas(),
            vmp = (stat.vsize / page_size).separate_with_commas(),
            rmp = stat.rss.separate_with_commas(),
            rmb = (stat.rss * page_size).separate_with_commas(),
        )
    } else {
        "- Couldn't access process information".to_string()
    }
}

#[cfg(not(target_os = "linux"))]
fn get_process_info() -> String {
    "- Process info only available on Linux hosts.".to_string()
}

impl BotCommand for UptimeCommand {
    fn prefix() -> &'static str {
        "/uptime"
    }

    fn description() -> &'static str {
        "Show bot uptime and statistics"
    }
}

impl Message<ActorUpdateMessage> for UptimeCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), _) = match_command(msg.update.text(), Self::prefix(), &self.bot_name) {
            let uptime = crate::datetime::format_uptime();
            let message = format!("- ⏰ Started {uptime}\n{}", get_process_info());
            self.send_reply(&msg, message).await;
        }
    }
}
