use crate::{
    services::{dreaming_city_cycle, escalation_protocol_cycle},
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
        let msg = format!(
            "This week in Destiny 2:\n\n{}\n\n{}",
            dreaming_city_cycle(),
            escalation_protocol_cycle()
        );
        bot.send_md_reply(&message, msg);
    }
}
