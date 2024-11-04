use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{admin_check, guardian_lookup, match_command},
        BotCommand,
    },
    riker::actors::Tell,
};

// #[derive(Clone)]
// struct ListAdminsSubcommand;
//
// #[derive(Clone)]
// struct AddAdminSubcommand;
//
// #[derive(Clone)]
// struct RemoveAdminSubcommand;

command_actor!(ManageCommand, [ActorUpdateMessage]);

impl ManageCommand {
    fn send_reply<S>(&self, message: &ActorUpdateMessage, reply: S)
    where
        S: Into<String>,
    {
        self.bot_ref.tell(
            SendMessageReply(reply.into(), message.clone(), Format::Plain, Notify::Off),
            None,
        );
    }

    fn usage(&self, message: &ActorUpdateMessage) {
        self.send_reply(
            message,
            "Manage admins:
/manage list-admins
    List existing admins
/manage add-admin <id|@telegram|PSN>
    Add existing guardian as an admin
/manage remove-admin <id|@telegram|PSN>
    Remove admin rights from guardian",
        );
    }
}

impl BotCommand for ManageCommand {
    fn prefix() -> &'static str {
        "/manage"
    }

    fn description() -> &'static str {
        "Manage bot users (admin-only)"
    }
}

impl Receive<ActorUpdateMessage> for ManageCommand {
    type Msg = ManageCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            let connection = self.connection();
            let admin = admin_check(&self.bot_ref, &message, &connection);

            if admin.is_none() {
                return self.send_reply(&message, "You are not admin");
            }

            // let _admin = admin.unwrap();

            if args.is_none() {
                return self.usage(&message);
            }

            // Split args in two:
            // subcommand,
            // and optional guardian id
            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(2, ' ').collect();

            if args.is_empty() {
                return self.usage(&message);
            }

            let subcommand = args[0];
            let args = if args.len() > 1 {
                Some(args[1].to_string())
            } else {
                None
            };

            log::info!("{:?}", args);

            match subcommand {
                "list-admins" => self.list_admins_subcommand(&message),
                "add-admin" => self.add_admin_subcommand(&message, args),
                "remove-admin" => self.remove_admin_subcommand(&message, args),
                &_ => {
                    self.send_reply(&message, "Unknown management command");
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
}

// command_ctor!(ListAdminsSubcommand);
//
// impl BotCommand for ListAdminsSubcommand {
//     fn prefix(&self) -> &'static str {
//         "list-admins"
//     }
//
//     fn description(&self) -> &'static str {
//         "List bot admins (admin-only)"
//     }
impl ManageCommand {
    fn list_admins_subcommand(&self, message: &ActorUpdateMessage) {
        use {
            crate::{models::Guardian, schema::guardians::dsl::*},
            diesel::prelude::*,
        };

        let connection = self.connection();

        let admins = guardians
            .filter(is_admin.eq(true))
            .order(telegram_name.asc())
            .load::<Guardian>(&connection)
            .expect("Cannot execute SQL query");

        if admins.is_empty() {
            return self.send_reply(message, "No admins found");
        }

        let text = admins
            .iter()
            .fold("Existing admins:\n\n".to_owned(), |acc, admin| {
                acc + &format!("{}\n", admin)
            });

        self.send_reply(message, text);
    }
}

// command_ctor!(AddAdminSubcommand);
//
// impl BotCommand for AddAdminSubcommand {
//     fn prefix(&self) -> &'static str {
//         "add-admin"
//     }
//
//     fn description(&self) -> &'static str {
//         "Add bot admin (admin-only)"
//     }
impl ManageCommand {
    fn add_admin_subcommand(&self, message: &ActorUpdateMessage, args: Option<String>) {
        let connection = self.connection();
        let admin = admin_check(&self.bot_ref, message, &connection);

        if admin.is_none() {
            return self.send_reply(message, "You are not admin");
        }

        let admin = admin.unwrap();

        if !admin.is_superadmin {
            return self.send_reply(message, "You are not superadmin");
        }

        if args.is_none() {
            return self.send_reply(message, "Specify a guardian to promote to admin");
        }

        let name = args.unwrap();

        let guardian = guardian_lookup(&name, &connection);

        match guardian {
            Ok(Some(mut guardian)) => {
                let tg_name = guardian.telegram_name.clone();

                if guardian.is_admin {
                    return self.send_reply(message, format!("@{} is already an admin", &tg_name));
                }

                use diesel_derives_traits::Model;

                guardian.is_admin = true;
                guardian
                    .save(&connection) // @todo handle DbError
                    .expect("Cannot execute SQL query");

                self.send_reply(message, format!("@{} is now an admin!", &tg_name));
            }
            Ok(None) => {
                self.send_reply(message, format!("Guardian {} was not found.", &name));
            }
            Err(_) => {
                self.send_reply(message, "Error querying guardian name.");
            }
        }
    }
}

// command_ctor!(RemoveAdminSubcommand);
//
// impl BotCommand for RemoveAdminSubcommand {
//     fn prefix(&self) -> &'static str {
//         "remove-admin"
//     }
//
//     fn description(&self) -> &'static str {
//         "Remove bot admin (admin-only)"
//     }

impl ManageCommand {
    fn remove_admin_subcommand(&self, message: &ActorUpdateMessage, args: Option<String>) {
        let connection = self.connection();
        let admin = admin_check(&self.bot_ref, message, &connection);

        if admin.is_none() {
            return self.send_reply(message, "You are not admin");
        }

        let admin = admin.unwrap();

        if !admin.is_superadmin {
            return self.send_reply(message, "You are not superadmin");
        }

        if args.is_none() {
            return self.send_reply(message, "Specify a guardian to demote from admins");
        }

        let name = args.unwrap();

        let guardian = guardian_lookup(&name, &connection);

        match guardian {
            Ok(Some(mut guardian)) => {
                let tg_name = guardian.telegram_name.clone();

                if !guardian.is_admin {
                    return self
                        .send_reply(message, format!("@{} is already not an admin", &tg_name));
                }

                if guardian.is_superadmin {
                    return self.send_reply(
                        message,
                        format!("@{} is a superadmin, you can not demote.", &tg_name),
                    );
                }

                use diesel_derives_traits::Model;

                guardian.is_admin = false;
                guardian
                    .save(&connection) // @todo handle DbError
                    .expect("Cannot execute SQL query");

                self.send_reply(message, format!("@{} is not an admin anymore!", &tg_name));
            }
            Ok(None) => {
                self.send_reply(message, format!("Guardian {} was not found.", &name));
            }
            Err(_) => {
                self.send_reply(message, "Error querying guardian name.");
            }
        }
    }
}
