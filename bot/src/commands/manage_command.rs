use {
    crate::{
        actors::bot_actor::ActorUpdateMessage,
        commands::{admin_check, guardian_lookup, match_command},
        render_template_or_err,
    },
    entity::guardians,
    kameo::message::Context,
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
        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            let connection = self.connection();

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
                "list-admins" => self.list_admins_subcommand(&message, connection).await,
                "add-admin" => self.add_admin_subcommand(&message, args, connection).await,
                "remove-admin" => {
                    self.remove_admin_subcommand(&message, args, connection)
                        .await
                }
                &_ => {
                    self.send_reply(&message, "❌ Unknown management command")
                        .await;
                }
            }
        }
    }
}

impl ManageCommand {
    async fn list_admins_subcommand(
        &self,
        message: &ActorUpdateMessage,
        connection: &DatabaseConnection,
    ) {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

        let admins = guardians::Entity::find()
            .filter(guardians::Column::IsAdmin.eq(true))
            .order_by_asc(guardians::Column::TelegramName)
            .all(connection)
            .await
            .expect("❌ Cannot execute SQL query");

        if admins.is_empty() {
            return self.send_reply(message, "❌ No admins found").await;
        }

        let admins: Vec<String> = admins.into_iter().map(|adm| adm.format_name()).collect();

        self.send_reply(
            message,
            render_template_or_err!("manage/admins", ("admins" => &admins)),
        )
        .await;
    }

    async fn add_admin_subcommand(
        &self,
        message: &ActorUpdateMessage,
        args: Option<String>,
        connection: &DatabaseConnection,
    ) {
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
                        .send_reply(message, format!("✅ @{tg_name} is already an admin"))
                        .await;
                }

                let mut guardian: guardians::ActiveModel = guardian.into();
                guardian.is_admin = Set(true);

                if guardian.update(connection).await.is_err() {
                    return self.send_reply(message, "❌ Error updating guardian").await;
                }

                self.send_reply(message, format!("✅ @{tg_name} is now an admin!"))
                    .await;
            }
            Ok(None) => {
                self.send_reply(message, format!("❌ Guardian {name} was not found."))
                    .await;
            }
            Err(_) => {
                self.send_reply(message, "❌ Error querying guardian name.")
                    .await;
            }
        }
    }

    async fn remove_admin_subcommand(
        &self,
        message: &ActorUpdateMessage,
        args: Option<String>,
        connection: &DatabaseConnection,
    ) {
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
                        .send_reply(message, format!("✅ @{tg_name} is already not an admin"))
                        .await;
                }

                if guardian.is_superadmin {
                    return self
                        .send_reply(
                            message,
                            format!("❌ @{tg_name} is a superadmin, you can not demote."),
                        )
                        .await;
                }

                let mut guardian: guardians::ActiveModel = guardian.into();
                guardian.is_admin = Set(false);

                if guardian.update(connection).await.is_err() {
                    return self.send_reply(message, "❌ Error updating guardian").await;
                }

                self.send_reply(message, format!("✅ @{tg_name} is not an admin anymore!"))
                    .await;
            }
            Ok(None) => {
                self.send_reply(message, format!("❌ Guardian {name} was not found."))
                    .await;
            }
            Err(_) => {
                self.send_reply(message, "❌ Error querying guardian name.")
                    .await;
            }
        }
    }
}
