use {
    crate::{
        actors::bot_actor::ActorUpdateMessage,
        commands::{admin_check, guardian_lookup, match_command},
    },
    entity::guardians,
    sea_orm::{ActiveModelTrait, Set},
};

command_actor!(ManageCommand, "manage", "Manage bot users (admin-only)");

impl Message<ActorUpdateMessage> for ManageCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let connection = self.connection();

        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            let admin = admin_check(&self.bot_ref, &message, connection).await;

            if admin.is_none() {
                return self.send_reply(&message, "❌ You are not admin").await;
            }

            // let _admin = admin.unwrap();

            if args.is_none() {
                return self.usage(&message).await;
            }

            // Split args in two:
            // subcommand,
            // and optional guardian id
            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(2, ' ').collect();

            if args.is_empty() {
                return self.usage(&message).await;
            }

            let subcommand = args[0];
            let args = if args.len() > 1 {
                Some(args[1].to_string())
            } else {
                None
            };

            log::info!("{:?}", args);

            match subcommand {
                "list-admins" => self.list_admins_subcommand(&message).await,
                "add-admin" => self.add_admin_subcommand(&message, args).await,
                "remove-admin" => self.remove_admin_subcommand(&message, args).await,
                &_ => {
                    self.send_reply(&message, "Unknown management command")
                        .await;
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
    async fn list_admins_subcommand(&self, message: &ActorUpdateMessage) {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

        let connection = self.connection();

        let admins = guardians::Entity::find()
            .filter(guardians::Column::IsAdmin.eq(true))
            .order_by_asc(guardians::Column::TelegramName)
            .all(connection)
            .await
            .expect("Cannot execute SQL query");

        if admins.is_empty() {
            return self.send_reply(message, "No admins found").await;
        }

        let text = admins
            .iter()
            .fold("Existing admins:\n\n".to_owned(), |acc, admin| {
                acc + &format!("{}\n", admin)
            });

        self.send_reply(message, text).await;
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
    async fn add_admin_subcommand(&self, message: &ActorUpdateMessage, args: Option<String>) {
        let connection = self.connection();
        let admin = admin_check(&self.bot_ref, message, connection).await;

        if admin.is_none() {
            return self.send_reply(message, "❌ You are not admin").await;
        }

        let admin = admin.unwrap();

        if !admin.is_superadmin {
            return self.send_reply(message, "❌ You are not superadmin").await;
        }

        if args.is_none() {
            return self
                .send_reply(message, "❌ Specify a guardian to promote to admin")
                .await;
        }

        let name = args.unwrap();

        let guardian = guardian_lookup(&name, connection).await;

        match guardian {
            Ok(Some(guardian)) => {
                let tg_name = guardian.telegram_name.clone();

                if guardian.is_admin {
                    return self
                        .send_reply(message, format!("✅ @{} is already an admin", &tg_name))
                        .await;
                }

                let mut guardian: guardians::ActiveModel = guardian.into();
                guardian.is_admin = Set(true);

                if guardian.update(connection).await.is_err() {
                    return self.send_reply(message, "Error updating guardian").await;
                }

                self.send_reply(message, format!("✅ @{} is now an admin!", &tg_name))
                    .await;
            }
            Ok(None) => {
                self.send_reply(message, format!("❌ Guardian {} was not found.", &name))
                    .await;
            }
            Err(_) => {
                self.send_reply(message, "❌ Error querying guardian name.")
                    .await;
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
    async fn remove_admin_subcommand(&self, message: &ActorUpdateMessage, args: Option<String>) {
        let connection = self.connection();
        let admin = admin_check(&self.bot_ref, message, connection).await;

        if admin.is_none() {
            return self.send_reply(message, "❌ You are not admin").await;
        }

        let admin = admin.unwrap();

        if !admin.is_superadmin {
            return self.send_reply(message, "❌ You are not superadmin").await;
        }

        if args.is_none() {
            return self
                .send_reply(message, "❌ Specify a guardian to demote from admins")
                .await;
        }

        let name = args.unwrap();

        let guardian = guardian_lookup(&name, connection).await;

        match guardian {
            Ok(Some(guardian)) => {
                let tg_name = guardian.telegram_name.clone();

                if !guardian.is_admin {
                    return self
                        .send_reply(message, format!("✅ @{} is already not an admin", &tg_name))
                        .await;
                }

                if guardian.is_superadmin {
                    return self
                        .send_reply(
                            message,
                            format!("❌ @{} is a superadmin, you can not demote.", &tg_name),
                        )
                        .await;
                }

                let mut guardian: guardians::ActiveModel = guardian.into();
                guardian.is_admin = Set(false);

                if guardian.update(connection).await.is_err() {
                    return self.send_reply(message, "Error updating guardian").await;
                }

                self.send_reply(
                    message,
                    format!("✅ @{} is not an admin anymore!", &tg_name),
                )
                .await;
            }
            Ok(None) => {
                self.send_reply(message, format!("Guardian {} was not found.", &name))
                    .await;
            }
            Err(_) => {
                self.send_reply(message, "Error querying guardian name.")
                    .await;
            }
        }
    }
}
