//☐ change guardian PSN name
//☐ `/editguar @alexundr psn Kayouga`
//☐ edit guardian clan
//☐ `/editguar Kayouga clan AEGL`
//☐ other fields in guardians table
//☐ `/editguar GUARDIAN_ID <field_name> <freeform value>`
//☐ GUARDIAN_ID could be int, telegram name or psn name
//☐ just show guardian fields
//☐ `/editguar GUARDIAN_ID`
use crate::{Bot, BotCommand, DbConnection};

pub struct EditGuardianCommand;

impl EditGuardianCommand {
    pub fn new() -> Box<Self> {
        Box::new(EditGuardianCommand)
    }
}

impl BotCommand for EditGuardianCommand {
    fn prefix(&self) -> &'static str {
        "/editguar"
    }

    fn description(&self) -> &'static str {
        "Edit information about registered guardians (admin-only)"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _name: Option<String>,
    ) {
        bot.send_plain_reply(&message, "Not implemented".to_string());
    }
}
