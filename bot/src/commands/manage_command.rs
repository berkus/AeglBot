use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{admin_check, guardian_lookup, match_command},
        BotCommand,
    },
    entity::guardians,
    riker::actors::Tell,
    sea_orm::{ActiveModelTrait, Set},
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

    fn send_reply_static<S>(
        bot_ref: &riker::actors::ActorRef<crate::bot_actor::BotActorMsg>,
        message: &ActorUpdateMessage,
        reply: S,
    ) where
        S: Into<String>,
    {
        bot_ref.tell(
            SendMessageReply(reply.into(), message.clone(), Format::Plain, Notify::Off),
            None,
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
        tokio::runtime::Handle::current().block_on(async {
            self.handle_message(message).await;
        });
    }
}

impl ManageCommand {
    async fn handle_message(&self, message: ActorUpdateMessage) {
        let connection = self.connection();

        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            let admin = admin_check(&self.bot_ref, &message, connection).await;

            if admin.is_none() {
                return self.send_reply(&message, "You are not admin");
            }

            // let _admin = admin.unwrap();

            if args.is_none() {
                return Self::send_usage(&self.bot_ref, &message);
            }

            // Split args in two:
            // subcommand,
            // and optional guardian id
            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(2, ' ').collect();

            if args.is_empty() {
                return Self::send_usage(&self.bot_ref, &message);
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
                    Self::list_admins_subcommand(&self.bot_ref, connection, &message).await
                }
                "add-admin" => {
                    Self::add_admin_subcommand(&self.bot_ref, connection, &message, args).await
                }
                "remove-admin" => {
                    Self::remove_admin_subcommand(&self.bot_ref, connection, &message, args).await
                }
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

    fn send_usage(
        bot_ref: &riker::actors::ActorRef<crate::bot_actor::BotActorMsg>,
        message: &ActorUpdateMessage,
    ) {
        Self::send_reply_static(
            bot_ref,
            message,
            "Manage command help:

/manage list-admins
    List currently registered admins.
/manage add-admin <id|@telegram|PSN>
    Grant admin rights to guardian.
/manage remove-admin <id|@telegram|PSN>
    Remove admin rights from guardian",
        );
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
    async fn list_admins_subcommand(
        bot_ref: &riker::actors::ActorRef<crate::bot_actor::BotActorMsg>,
        connection: &sea_orm::DatabaseConnection,
        message: &ActorUpdateMessage,
    ) {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

        let admins = guardians::Entity::find()
            .filter(guardians::Column::IsAdmin.eq(true))
            .order_by_asc(guardians::Column::TelegramName)
            .all(connection)
            .await
            .expect("Cannot execute SQL query");

        if admins.is_empty() {
            return Self::send_reply_static(bot_ref, message, "No admins found");
        }

        let text = admins
            .iter()
            .fold("Existing admins:\n\n".to_owned(), |acc, admin| {
                acc + &format!("{}\n", admin)
            });

        Self::send_reply_static(bot_ref, message, text);
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
    async fn add_admin_subcommand(
        bot_ref: &riker::actors::ActorRef<crate::bot_actor::BotActorMsg>,
        connection: &sea_orm::DatabaseConnection,
        message: &ActorUpdateMessage,
        args: Option<String>,
    ) {
        let admin = admin_check(bot_ref, message, connection).await;

        if admin.is_none() {
            return Self::send_reply_static(bot_ref, message, "You are not admin");
        }

        let admin = admin.unwrap();

        if !admin.is_superadmin {
            return Self::send_reply_static(bot_ref, message, "You are not superadmin");
        }

        if args.is_none() {
            return Self::send_reply_static(
                bot_ref,
                message,
                "Specify a guardian to promote to admin",
            );
        }

        let name = args.unwrap();

        let guardian = guardian_lookup(&name, connection).await;

        match guardian {
            Ok(Some(guardian)) => {
                let tg_name = guardian.telegram_name.clone();

                if guardian.is_admin {
                    return Self::send_reply_static(
                        bot_ref,
                        message,
                        format!("@{} is already an admin", &tg_name),
                    );
                }

                let mut guardian: guardians::ActiveModel = guardian.into();
                guardian.is_admin = Set(true);

                if guardian.update(connection).await.is_err() {
                    return Self::send_reply_static(bot_ref, message, "Error updating guardian");
                }

                Self::send_reply_static(
                    bot_ref,
                    message,
                    format!("@{} is now an admin!", &tg_name),
                );
            }
            Ok(None) => {
                Self::send_reply_static(
                    bot_ref,
                    message,
                    format!("Guardian {} was not found.", &name),
                );
            }
            Err(_) => {
                Self::send_reply_static(bot_ref, message, "Error querying guardian name.");
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
    async fn remove_admin_subcommand(
        bot_ref: &riker::actors::ActorRef<crate::bot_actor::BotActorMsg>,
        connection: &sea_orm::DatabaseConnection,
        message: &ActorUpdateMessage,
        args: Option<String>,
    ) {
        let admin = admin_check(bot_ref, message, connection).await;

        if admin.is_none() {
            return Self::send_reply_static(bot_ref, message, "You are not admin");
        }

        let admin = admin.unwrap();

        if !admin.is_superadmin {
            return Self::send_reply_static(bot_ref, message, "You are not superadmin");
        }

        if args.is_none() {
            return Self::send_reply_static(
                bot_ref,
                message,
                "Specify a guardian to demote from admins",
            );
        }

        let name = args.unwrap();

        let guardian = guardian_lookup(&name, connection).await;

        match guardian {
            Ok(Some(guardian)) => {
                let tg_name = guardian.telegram_name.clone();

                if !guardian.is_admin {
                    return Self::send_reply_static(
                        bot_ref,
                        message,
                        format!("@{} is already not an admin", &tg_name),
                    );
                }

                if guardian.is_superadmin {
                    return Self::send_reply_static(
                        bot_ref,
                        message,
                        format!("@{} is a superadmin, you can not demote.", &tg_name),
                    );
                }

                let mut guardian: guardians::ActiveModel = guardian.into();
                guardian.is_admin = Set(false);

                if guardian.update(connection).await.is_err() {
                    return Self::send_reply_static(bot_ref, message, "Error updating guardian");
                }

                Self::send_reply_static(
                    bot_ref,
                    message,
                    format!("@{} is not an admin anymore!", &tg_name),
                );
            }
            Ok(None) => {
                Self::send_reply_static(
                    bot_ref,
                    message,
                    format!("Guardian {} was not found.", &name),
                );
            }
            Err(_) => {
                Self::send_reply_static(bot_ref, message, "Error querying guardian name.");
            }
        }
    }
}
