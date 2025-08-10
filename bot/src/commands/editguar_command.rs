use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{admin_check, guardian_lookup, match_command, validate_username},
        render_template, BotCommand,
    },
    entity::guardians,
    riker::actors::Tell,
    sea_orm::{ActiveModelTrait, Set},
};

command_actor!(EditGuardianCommand, [ActorUpdateMessage]);

impl EditGuardianCommand {
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
            render_template!("editguar/usage").expect("Failed to render editguar usage template"),
        );
    }
}

impl BotCommand for EditGuardianCommand {
    fn prefix() -> &'static str {
        "/editguar"
    }

    fn description() -> &'static str {
        "Edit information about registered guardians"
    }
}

impl Receive<ActorUpdateMessage> for EditGuardianCommand {
    type Msg = EditGuardianCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        tokio::runtime::Handle::current().block_on(async {
            self.handle_message(message).await;
        });
    }
}

impl EditGuardianCommand {
    async fn handle_message(&self, message: ActorUpdateMessage) {
        let connection = self.connection();

        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if args.is_none() {
                return self.usage(&message);
            }

            // Split args in two or three:
            // guardian id,
            // subcommand,
            // and optionally, parameters
            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(3, ' ').collect();

            if args.is_empty() || args.len() == 2 {
                return self.usage(&message);
            }

            let name = args[0];

            let guardian = if name == "my" {
                let guardian = validate_username(&self.bot_ref, &message, connection).await;
                if guardian.is_none() {
                    return;
                }
                guardian.unwrap()
            } else {
                let admin = admin_check(&self.bot_ref, &message, connection).await;

                if admin.is_none() {
                    return self.send_reply(&message, "You are not admin");
                }

                let guardian = guardian_lookup(name, connection).await;
                let guardian = match guardian {
                    Ok(Some(guardian)) => Some(guardian),
                    Ok(None) => {
                        self.send_reply(&message, format!("Guardian {} was not found.", &name));
                        None
                    }
                    Err(_) => {
                        self.send_reply(&message, "Error querying guardian by name.");
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
                return self.send_reply(&message, info);
            }

            let command = args[1];
            let value = args[2];

            match command {
                "psn" => {
                    let mut guardian: guardians::ActiveModel = guardian.into();
                    guardian.psn_name = Set(value.to_string());
                    if guardian.update(connection).await.is_err() {
                        return self.send_reply(&message, "Failed to update PSN");
                    }
                    self.send_reply(&message, "PSN updated successfully");
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
                        return self.send_reply(&message, "Failed to update clan");
                    }
                    self.send_reply(&message, "Updated guardian clan");
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
                        return self.send_reply(&message, "Failed to update email");
                    }
                    self.send_reply(&message, "Updated guardian email");
                }
                _ => {
                    self.send_reply(&message, "Unknown information field");
                }
            }
        }
    }
}
