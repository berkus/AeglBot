use {
    crate::{
        actors::bot_actor::ActorUpdateMessage,
        commands::{admin_check, guardian_lookup, match_command, validate_username},
    },
    entity::guardians,
    kameo::message::Context,
    sea_orm::{ActiveModelTrait, Set},
};

command_actor!(
    EditGuardianCommand,
    "editguar",
    "Edit information about registered guardians"
);

impl Message<ActorUpdateMessage> for EditGuardianCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if args.is_none() {
                return self.usage(&message).await;
            }

            // Split args in two or three:
            // guardian id,
            // subcommand,
            // and optionally, parameters
            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(3, ' ').collect();

            if args.is_empty() || args.len() == 2 {
                return self.usage(&message).await;
            }

            let name = args[0];

            let connection = self.connection();

            let guardian = if name == "my" {
                let guardian = validate_username(&self.bot_ref, &message, connection).await;
                if guardian.is_none() {
                    return; // TODO: You are not registered
                }
                guardian.unwrap()
            } else {
                let admin = admin_check(&self.bot_ref, &message, connection).await;

                if admin.is_none() {
                    return self.send_reply(&message, "❌ You are not admin").await;
                }

                let guardian = guardian_lookup(name, connection).await;
                let guardian = match guardian {
                    Ok(Some(guardian)) => Some(guardian),
                    Ok(None) => {
                        self.send_reply(&message, format!("❌ Guardian {} was not found.", &name))
                            .await;
                        None
                    }
                    Err(_) => {
                        self.send_reply(&message, "❌ Error querying guardian by name.")
                            .await;
                        None
                    }
                };
                if guardian.is_none() {
                    return;
                }
                guardian.unwrap()
            };

            if args.len() == 1 {
                let info = format!(
                    "{clan}{name} {email} {admin}",
                    clan = guardian
                        .psn_clan
                        .clone()
                        .map(|s| format!("[{}] ", s))
                        .unwrap_or_default(),
                    name = guardian.telegram_name,
                    email = guardian.email.clone().unwrap_or("<no email>".into()),
                    admin = if guardian.is_superadmin {
                        "<superadmin>"
                    } else if guardian.is_admin {
                        "<admin>"
                    } else {
                        ""
                    },
                );
                return self.send_reply(&message, info).await;
            }

            let command = args[1];
            let value = args[2];

            match command {
                "psn" => {
                    let mut guardian: guardians::ActiveModel = guardian.into();
                    guardian.psn_name = Set(value.to_string());
                    if guardian.update(connection).await.is_err() {
                        return self.send_reply(&message, "❌ Failed to update PSN").await;
                    }
                    self.send_reply(&message, "✅ PSN updated successfully")
                        .await;
                }
                "clan" => {
                    let clan_value = if value == "delete" {
                        None
                    } else {
                        Some(value.to_string())
                    };
                    let mut guardian: guardians::ActiveModel = guardian.into();
                    guardian.psn_clan = Set(clan_value);
                    if guardian.update(connection).await.is_err() {
                        return self.send_reply(&message, "❌ Failed to update clan").await;
                    }
                    self.send_reply(&message, "✅ Updated guardian clan").await;
                }
                "email" => {
                    let email_value = if value == "delete" {
                        None
                    } else {
                        Some(value.to_string())
                    };
                    let mut guardian: guardians::ActiveModel = guardian.into();
                    guardian.email = Set(email_value);
                    if guardian.update(connection).await.is_err() {
                        return self.send_reply(&message, "❌ Failed to update email").await;
                    }
                    self.send_reply(&message, "✅ Updated guardian email").await;
                }
                _ => {
                    self.send_reply(&message, "⁉️ Unknown information field")
                        .await;
                }
            }
        }
    }
}
