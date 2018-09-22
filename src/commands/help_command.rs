use crate::{Bot, BotCommand, DbConnection};

pub struct HelpCommand;

command_ctor!(HelpCommand);

impl BotCommand for HelpCommand {
    fn prefix(&self) -> &'static str {
        "/help"
    }

    fn description(&self) -> &'static str {
        "List available commands"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _name: Option<String>,
    ) {
        let mut sorted_cmds = bot.list_commands();
        sorted_cmds.sort_by_cached_key(|v| v.0.to_owned());

        bot.send_html_reply(
            &message,
            sorted_cmds.into_iter().fold(
                "<b>Help</b> 🚑\nThese are the registered commands for this Bot:\n\n".into(),
                |acc, pair| format!("{}{} — {}\n\n", acc, pair.0, pair.1),
            ),
        );
    }
}
