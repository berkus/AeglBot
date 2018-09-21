//☐ /edit command to modify activities
//☐ `/edit530 time 11:00`
//☐ `/edit530 desc <new description>`
//☐ `/edit530 activity <new activity shortcut>`
use crate::{Bot, BotCommand, DbConnection};

pub struct EditCommand;

impl EditCommand {
    pub fn new() -> Box<Self> {
        Box::new(EditCommand)
    }
}

impl BotCommand for EditCommand {
    fn prefix(&self) -> &'static str {
        "/edit"
    }

    fn description(&self) -> &'static str {
        "Edit existing activity"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _args: Option<String>,
    ) {
        bot.send_plain_reply(&message, "Not implemented".to_string());
    }
}
