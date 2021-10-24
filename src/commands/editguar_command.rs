use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{admin_check, guardian_lookup, match_command, validate_username},
        BotCommand,
    },
    riker::actors::Tell,
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
            "Edit guardian information:
/editguar <id|@telegram|PSN|'my'>
    List known guardian information
/editguar <id|@telegram|PSN|'my'> psn <NewPSN>
    Change guardian's PSN
/editguar <id|@telegram|PSN|'my'> clan <Clan ticker, e.g. AEGL or \"delete\">
    Change guardian's clan
/editguar <id|@telegram|PSN|'my'> email <NewEmail>
    Change guardian's email",
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
        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            let connection = self.connection();

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
                let guardian = validate_username(&self.bot_ref, &message, &connection);
                if guardian.is_none() {
                    return;
                }
                guardian.unwrap()
            } else {
                let admin = admin_check(&self.bot_ref, &message, &connection);

                if admin.is_none() {
                    return self.send_reply(&message, "You are not admin");
                }

                let guardian = guardian_lookup(&name, &connection);
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
                    clan = if guardian.psn_clan.is_none() {
                        "".into()
                    } else {
                        format!("[{}] ", guardian.psn_clan.clone().unwrap())
                    },
                    name = guardian.format_name(),
                    email = if guardian.email.is_none() {
                        "<no email>".into()
                    } else {
                        guardian.email.clone().unwrap().to_string()
                    },
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

            let mut guardian = guardian;

            use diesel_derives_traits::Model;

            match command {
                "psn" => {
                    guardian.psn_name = value.into();
                    guardian.save(&connection).expect("Failed to update PSN");
                    self.send_reply(&message, "Updated guardian PSN");
                }
                "clan" => {
                    let value = if value == "delete" {
                        None
                    } else {
                        Some(value.into())
                    };
                    guardian.psn_clan = value;
                    guardian.save(&connection).expect("Failed to update clan");
                    self.send_reply(&message, "Updated guardian clan");
                }
                "email" => {
                    let value = if value == "delete" {
                        None
                    } else {
                        Some(value.into())
                    };
                    guardian.email = value;
                    guardian.save(&connection).expect("Failed to update email");
                    self.send_reply(&message, "Updated guardian email");
                }
                _ => {
                    self.send_reply(&message, "Unknown information field");
                }
            }
        }
    }
}
