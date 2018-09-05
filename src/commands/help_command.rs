use crate::commands::{send_html_reply, BotCommand};
use crate::DbConnection;
use telebot::RcBot;

pub struct HelpCommand;

impl BotCommand for HelpCommand {
    fn prefix() -> &'static str {
        "help"
    }

    fn description() -> &'static str {
        "List available commands"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        _name: Option<String>,
        _connection: &DbConnection,
    ) {
        //         commandRegistry.getRegisteredCommands().forEach { botCommand: BotCommand ->
        //             helpMessageBuilder.append(botCommand.toString()).append("\n\n")
        //         }
        send_html_reply(
            bot,
            &message,
            "<b>Help</b> 🚑\nThese are the registered commands for this Bot:\n\n".into(),
        );
    }
}
