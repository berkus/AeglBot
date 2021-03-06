use crate::{
    services::{this_week_in_d2},
    {Bot, BotCommand, DbConnection},
};

pub struct D2weekCommand;

command_ctor!(D2weekCommand);

impl BotCommand for D2weekCommand {
    fn prefix(&self) -> &'static str {
        "/d2week"
    }

    fn description(&self) -> &'static str {
        "Show current Destiny 2 week"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _name: Option<String>,
    ) {
        bot.send_md_reply(&message, this_week_in_d2());
    }
}
