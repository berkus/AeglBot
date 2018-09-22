//☐ change guardian PSN name
//☐ `/editguar @alexundr psn Kayouga`
//☐ edit guardian clan
//☐ `/editguar Kayouga clan AEGL`
//☐ other fields in guardians table
//☐ `/editguar GUARDIAN_ID <field_name> <freeform value>`
//☐ GUARDIAN_ID could be int, telegram name or psn name
//☐ just show guardian fields
//☐ `/editguar GUARDIAN_ID`
/// Allow editing info about yourself
use crate::{commands::admin_check, Bot, BotCommand, DbConnection};

pub struct EditGuardianCommand;

impl EditGuardianCommand {
    pub fn new() -> Box<Self> {
        Box::new(EditGuardianCommand)
    }

    fn usage(bot: &Bot, message: &telebot::objects::Message) {
        bot.send_plain_reply(
            &message,
            "Edit guardian information:
/editguar <id|@telegram|PSN>
    List known guardian information
/editguar <id|@telegram|PSN> psn <NewPSN>
    Change guardian's PSN
/editguar <id|@telegram|PSN> clan <Clan ticker, e.g. AEGL>
    Change guardian's clan
/editguar <id|@telegram|PSN> email <NewEmail>
    Change guardian's email"
                .into(),
        );
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
        args: Option<String>,
    ) {
        let connection = bot.connection();
        let admin = admin_check(bot, message, &connection);

        if admin.is_none() {
            return bot.send_plain_reply(&message, "You are not admin".to_string());
        }

        let _admin = admin.unwrap();

        if args.is_none() {
            return EditGuardianCommand::usage(bot, &message);
        }

        // Split args in two or three:
        // guardian id,
        // subcommand,
        // and optionally, parameters
        let args = args.unwrap();
        let args: Vec<&str> = args.splitn(2, ' ').collect();

        if args.len() < 2 {
            return EditGuardianCommand::usage(bot, &message);
        }

        info!("{:?}", args);

        bot.send_plain_reply(&message, "Not implemented".into());
    }
}
