use crate::{Bot, BotCommand, DbConnection};

pub struct HelpCommand;

impl BotCommand for HelpCommand {
    fn prefix(&self) -> &'static str {
        "help"
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
        //         commandRegistry.getRegisteredCommands().forEach { botCommand: BotCommand ->
        //             helpMessageBuilder.append(botCommand.toString()).append("\n\n")
        //         }
        bot.send_html_reply(
            &message,
            "<b>Help</b> 🚑\nThese are the registered commands for this Bot:\n\n".into(),
        );
    }
}
