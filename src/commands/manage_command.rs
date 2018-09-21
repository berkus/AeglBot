//☐ manage admins (superadmin can add/remove admins, admins cannot add more admins?)
//☐ `/manage` catch-all command for these things
//* `list-admins`, `add-admin`, `remove-admin` subcommands
use crate::{Bot, BotCommand, DbConnection};

pub struct ManageCommand;
struct ListAdminsSubcommand;
struct AddAdminSubcommand;
struct RemoveAdminSubcommand;

impl ManageCommand {
    pub fn new() -> Box<Self> {
        Box::new(ManageCommand)
    }
}

impl BotCommand for ManageCommand {
    fn prefix(&self) -> &'static str {
        "/manage"
    }

    fn description(&self) -> &'static str {
        "Manage bot users (admin-only)"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _name: Option<String>,
    ) {
        // Need to invent some sort of match string format for matching subcommands
        // Some are `/command subcommand [args]`, some are `/command arg subcommand args` etc.
        // Can encode this string in prefix() for subcommands and make them match, maybe even directly?
        // i.e. add subcommands together with master command to the general list of commands (need to sort properly too)
        //        if match_subcommand(message, ListAdminsSubcommand) {
        //            return ListAdminsSubcommand::execute();
        //        } else if match_subcommand(message, AddAdminSubcommand) {
        //            return AddAdminSubcommand::execute();
        //        }
        bot.send_plain_reply(&message, "Not implemented".to_string());
    }
}

impl BotCommand for ListAdminsSubcommand {
    fn prefix(&self) -> &'static str {
        "list-admins"
    }

    fn description(&self) -> &'static str {
        "List bot admins (admin-only)"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _name: Option<String>,
    ) {
    }
}

impl BotCommand for AddAdminSubcommand {
    fn prefix(&self) -> &'static str {
        "add-admin"
    }

    fn description(&self) -> &'static str {
        "Add bot admin (admin-only)"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _name: Option<String>,
    ) {
    }
}

impl BotCommand for RemoveAdminSubcommand {
    fn prefix(&self) -> &'static str {
        "remove-admin"
    }

    fn description(&self) -> &'static str {
        "Remove bot admin (admin-only)"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _name: Option<String>,
    ) {
    }
}
