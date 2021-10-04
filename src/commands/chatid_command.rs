use {
    crate::{BotCommand, BotMenu, DbConnection},
    teloxide::prelude::*,
};

pub struct ChatidCommand;

command_ctor!(ChatidCommand);

impl BotCommand for ChatidCommand {
    fn prefix(&self) -> &'static str {
        "/chatid"
    }

    fn description(&self) -> &'static str {
        "Figure out the numeric chat ID"
    }

    fn execute(
        &self,
        bot: &BotMenu,
        message: &UpdateWithCx<AutoSend<Bot>, Message>,
        _command: Option<String>,
        _name: Option<String>,
    ) {
        bot.send_html_reply(&message, format!("ChatId: {}", message.chat_id()));
    }
}
