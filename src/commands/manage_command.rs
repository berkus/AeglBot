use {
    crate::{commands::admin_check, commands::guardian_lookup, BotCommand, BotMenu, DbConnection},
    teloxide::prelude::*,
};

pub struct ManageCommand;
struct ListAdminsSubcommand;
struct AddAdminSubcommand;
struct RemoveAdminSubcommand;

command_ctor!(ManageCommand);

impl ManageCommand {
    fn usage(bot: &BotMenu, message: &UpdateWithCx<AutoSend<Bot>, Message>) {
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
        bot: &BotMenu,
        message: &UpdateWithCx<AutoSend<Bot>, Message>,
        _command: Option<String>,
        args: Option<String>,
    ) {
        let connection = bot.connection();
        let admin = admin_check(bot, message, &connection);

        if admin.is_none() {
            return bot.send_plain_reply(&message, "You are not admin".into());
        }

        let _admin = admin.unwrap();

        if args.is_none() {
            return ManageCommand::usage(bot, &message);
        }

        // Split args in two:
        // subcommand,
        // and optional guardian id
        let args = args.unwrap();
        let args: Vec<&str> = args.splitn(2, ' ').collect();

        if args.is_empty() {
            return ManageCommand::usage(bot, &message);
        }

        let subcommand = args[0];
        let args = if args.len() > 1 {
            Some(args[1].to_string())
        } else {
            None
        };

        log::info!("{:?}", args);

        match subcommand {
            "list-admins" => {
                ListAdminsSubcommand::new().execute(bot, message, Some("list-admins".into()), None)
            }
            "add-admin" => {
                AddAdminSubcommand::new().execute(bot, message, Some("add-admin".into()), args)
            }
            "remove-admin" => RemoveAdminSubcommand::new().execute(
                bot,
                message,
                Some("remove-admin".into()),
                args,
            ),
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

command_ctor!(ListAdminsSubcommand);

impl BotCommand for ListAdminsSubcommand {
    fn prefix(&self) -> &'static str {
        "list-admins"
    }

    fn description(&self) -> &'static str {
        "List bot admins (admin-only)"
    }

    fn execute(
        &self,
        bot: &BotMenu,
        message: &UpdateWithCx<AutoSend<Bot>, Message>,
        _command: Option<String>,
        _args: Option<String>,
    ) {
        use crate::{models::Guardian, schema::guardians::dsl::*};
        use diesel::prelude::*;

        let connection = bot.connection();

        let admins = guardians
            .filter(is_admin.eq(true))
            .order(telegram_name.asc())
            .load::<Guardian>(&connection)
            .expect("Cannot execute SQL query");

        if admins.is_empty() {
            bot.send_plain_reply(&message, "No admins found".into());
        } else {
            let text = admins
                .iter()
                .fold("Existing admins:\n\n".to_owned(), |acc, admin| {
                    acc + &format!("{}\n", admin)
                });

            bot.send_plain_reply(&message, text);
        }
    }
}

command_ctor!(AddAdminSubcommand);

impl BotCommand for AddAdminSubcommand {
    fn prefix(&self) -> &'static str {
        "add-admin"
    }

    fn description(&self) -> &'static str {
        "Add bot admin (admin-only)"
    }

    fn execute(
        &self,
        bot: &BotMenu,
        message: &UpdateWithCx<AutoSend<Bot>, Message>,
        _command: Option<String>,
        args: Option<String>,
    ) {
        let connection = bot.connection();
        let admin = admin_check(bot, message, &connection);

        if admin.is_none() {
            return bot.send_plain_reply(&message, "You are not admin".into());
        }

        let admin = admin.unwrap();

        if !admin.is_superadmin {
            return bot.send_plain_reply(&message, "You are not superadmin".into());
        }

        if args.is_none() {
            return bot
                .send_plain_reply(&message, "Specify a guardian to promote to admins".into());
        }

        let name = args.unwrap();

        let guardian = guardian_lookup(&name, &connection);

        match guardian {
            Ok(Some(mut guardian)) => {
                let tg_name = guardian.telegram_name.clone();

                if guardian.is_admin {
                    return bot
                        .send_plain_reply(&message, format!("@{} is already an admin", &tg_name));
                }

                use diesel_derives_traits::Model;

                guardian.is_admin = true;
                guardian
                    .save(&connection) // @todo handle DbError
                    .expect("Cannot execute SQL query");

                bot.send_plain_reply(&message, format!("@{} is now an admin!", &tg_name));
            }
            Ok(None) => {
                bot.send_plain_reply(&message, format!("Guardian {} was not found.", &name));
            }
            Err(_) => {
                bot.send_plain_reply(&message, "Error querying guardian name.".into());
            }
        }
    }
}

command_ctor!(RemoveAdminSubcommand);

impl BotCommand for RemoveAdminSubcommand {
    fn prefix(&self) -> &'static str {
        "remove-admin"
    }

    fn description(&self) -> &'static str {
        "Remove bot admin (admin-only)"
    }

    fn execute(
        &self,
        bot: &BotMenu,
        message: &UpdateWithCx<AutoSend<Bot>, Message>,
        _command: Option<String>,
        args: Option<String>,
    ) {
        let connection = bot.connection();
        let admin = admin_check(bot, message, &connection);

        if admin.is_none() {
            return bot.send_plain_reply(&message, "You are not admin".into());
        }

        let admin = admin.unwrap();

        if !admin.is_superadmin {
            return bot.send_plain_reply(&message, "You are not superadmin".into());
        }

        if args.is_none() {
            return bot
                .send_plain_reply(&message, "Specify a guardian to demote from admins".into());
        }

        let name = args.unwrap();

        let guardian = guardian_lookup(&name, &connection);

        match guardian {
            Ok(Some(mut guardian)) => {
                let tg_name = guardian.telegram_name.clone();

                if !guardian.is_admin {
                    return bot.send_plain_reply(
                        &message,
                        format!("@{} is already not an admin", &tg_name),
                    );
                }

                if guardian.is_superadmin {
                    return bot.send_plain_reply(
                        &message,
                        format!("@{} is a superadmin, you can not demote.", &tg_name),
                    );
                }

                use diesel_derives_traits::Model;

                guardian.is_admin = false;
                guardian
                    .save(&connection) // @todo handle DbError
                    .expect("Cannot execute SQL query");

                bot.send_plain_reply(&message, format!("@{} is not an admin anymore!", &tg_name));
            }
            Ok(None) => {
                bot.send_plain_reply(&message, format!("Guardian {} was not found.", &name));
            }
            Err(_) => {
                bot.send_plain_reply(&message, "Error querying guardian name.".into());
            }
        }
    }
}
