//☐ manage admins (superadmin can add/remove admins, admins cannot add more admins?)
//☐ `/manage` catch-all command for these things
use crate::{commands::admin_check, Bot, BotCommand, DbConnection};

pub struct ManageCommand;
struct ListAdminsSubcommand;
struct AddAdminSubcommand;
struct RemoveAdminSubcommand;

macro_rules! ctor {
    ($name:ident) => (impl $name {
        pub fn new() -> Box<Self> {
            Box::new($name)
        }
    })
}

ctor!(ManageCommand);

impl ManageCommand {
    fn usage(bot: &Bot, message: &telebot::objects::Message) {
        bot.send_plain_reply(
            &message,
            "Manage admins:
/manage list-admins
    List existing admins
/manage add-admin <id|@telegram|PSN>
    Add existing guardian as an admin
/manage remove-admin <id|@telegram|PSN>
    Remove admin rights from guardian"
                .into(),
        );
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
        args: Option<String>,
    ) {
        let connection = bot.connection();
        let admin = admin_check(bot, message, &connection);

        if admin.is_none() {
            return bot.send_plain_reply(&message, "You are not admin".to_string());
        }

        let admin = admin.unwrap();

        if args.is_none() {
            return ManageCommand::usage(bot, &message);
        }

        // Split args in two:
        // subcommand,
        // and optional guardian id
        let args = args.unwrap();
        let args: Vec<&str> = args.splitn(2, ' ').collect();

        if args.len() < 1 {
            return ManageCommand::usage(bot, &message);
        }

        let subcommand = args[0];
        let args = if args.len() > 1 { Some(args[1].to_string()) } else { None };

        info!("{:?}", args);

        match subcommand {
            "list-admins" => {
                ListAdminsSubcommand::new().execute(bot, message, Some("list-admins".into()), None)
            }
            "add-admin" => {
                AddAdminSubcommand::new().execute(bot, message, Some("add-admin".into()), args)
            }
            "remove-admin" => {
                RemoveAdminSubcommand::new().execute(bot, message, Some("remove-admin".into()), args)
            }
            &_ => {
                bot.send_plain_reply(&message, "Unknown management command".into());
            }
        }

        // Need to invent some sort of match string format for matching subcommands
        // Some are `/command subcommand [args]`, some are `/command arg subcommand args` etc.
        // Can encode this string in prefix() for subcommands and make them match, maybe even directly?
        // i.e. add subcommands together with master command to the general list of commands (need to sort properly too)
        //        if match_subcommand(message, ListAdminsSubcommand) {
        //            return ListAdminsSubcommand::execute();
        //        } else if match_subcommand(message, AddAdminSubcommand) {
        //            return AddAdminSubcommand::execute();
        //        }
    }
}

ctor!(ListAdminsSubcommand);

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
        args: Option<String>,
    ) {
        bot.send_plain_reply(&message, "List-admins command".into());
    }
}

ctor!(AddAdminSubcommand);

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
        args: Option<String>,
    ) {
        bot.send_plain_reply(&message, "Add-admin command".into());
    }
}

ctor!(RemoveAdminSubcommand);

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
        args: Option<String>,
    ) {
        bot.send_plain_reply(&message, "Remove-admin command".into());
    }
}
