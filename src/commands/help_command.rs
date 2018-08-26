use crate::commands::BotCommand;
use diesel::PgConnection;
use telegram_bot::{self, types::ParseMode, CanReplySendMessage};

pub struct HelpCommand;

impl BotCommand for HelpCommand {
    fn prefix() -> &'static str {
        "help"
    }

    fn description() -> &'static str {
        "List available commands"
    }

    fn execute(
        api: &telegram_bot::Api,
        message: &telegram_bot::Message,
        command: Option<String>,
        name: Option<String>,
        connection: &PgConnection,
    ) {
        //         commandRegistry.getRegisteredCommands().forEach { botCommand: BotCommand ->
        //             helpMessageBuilder.append(botCommand.toString()).append("\n\n")
        //         }
        api.spawn(
            message
                .text_reply("<b>Help</b> 🚑\nThese are the registered commands for this Bot:\n\n")
                .parse_mode(ParseMode::Html),
        );
    }
}
