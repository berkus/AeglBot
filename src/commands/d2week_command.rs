use {
    crate::{
        services::this_week_in_d2,
        {BotCommand, BotMenu, DbConnection, UpdateMessage},
    },
    teloxide::prelude::*,
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
        bot: &BotMenu,
        message: &UpdateMessage,
        _command: Option<String>,
        _name: Option<String>,
    ) {
        bot.send_md_reply(&message, this_week_in_d2());
    }
}
