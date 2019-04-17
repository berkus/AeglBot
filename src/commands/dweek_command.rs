use {
    crate::{
        services::this_week_in_d1,
        {BotCommand, BotMenu, DbConnection, UpdateMessage},
    },
    teloxide::prelude::*,
};

pub struct D1weekCommand;

command_ctor!(D1weekCommand);

impl BotCommand for D1weekCommand {
    fn prefix(&self) -> &'static str {
        "/dweek"
    }

    fn description(&self) -> &'static str {
        "Show current Destiny 1 week"
    }

    fn execute(
        &self,
        bot: &BotMenu,
        message: &UpdateMessage,
        _command: Option<String>,
        _name: Option<String>,
    ) {
        bot.send_md_reply(&message, this_week_in_d1());
    }
}
