use {
    crate::{
        commands::admin_check,
        commands::{guardian_lookup, validate_username},
        BotCommand, BotMenu, DbConnection,
    },
    teloxide::prelude::*,
};

#[derive(Clone)]
pub struct EditGuardianCommand;

command_ctor!(EditGuardianCommand);

impl EditGuardianCommand {
    fn usage(bot: &BotMenu, message: &UpdateWithCx<AutoSend<Bot>, Message>) {
        bot.send_plain_reply(
            &message,
            "Edit guardian information:
/editguar <id|@telegram|PSN|'my'>
    List known guardian information
/editguar <id|@telegram|PSN|'my'> psn <NewPSN>
    Change guardian's PSN
/editguar <id|@telegram|PSN|'my'> clan <Clan ticker, e.g. AEGL>
    Change guardian's clan
/editguar <id|@telegram|PSN|'my'> email <NewEmail>
    Change guardian's email"
                .into(),
        );
    }
}

impl BotCommand for EditGuardianCommand {
    fn prefix(&self) -> &'static str {
        "/editguar"
    }

    fn description(&self) -> &'static str {
        "Edit information about registered guardians"
    }

    fn execute(
        &self,
        bot: &BotMenu,
        message: &UpdateWithCx<AutoSend<Bot>, Message>,
        _command: Option<String>,
        args: Option<String>,
    ) {
        let connection = bot.connection();

        if args.is_none() {
            return EditGuardianCommand::usage(bot, &message);
        }

        // Split args in two or three:
        // guardian id,
        // subcommand,
        // and optionally, parameters
        let args = args.unwrap();
        let args: Vec<&str> = args.splitn(3, ' ').collect();

        if args.is_empty() || args.len() == 2 {
            return EditGuardianCommand::usage(bot, &message);
        }

        let name = args[0];

        let guardian = if name == "my" {
            let guardian = validate_username(bot, message, &connection);
            if guardian.is_none() {
                return;
            }
            guardian.unwrap()
        } else {
            let admin = admin_check(bot, message, &connection);

            if admin.is_none() {
                return bot.send_plain_reply(&message, "You are not admin".to_string());
            }

            let guardian = guardian_lookup(&name, &connection);
            let guardian = match guardian {
                Ok(Some(guardian)) => Some(guardian),
                Ok(None) => {
                    bot.send_plain_reply(&message, format!("Guardian {} was not found.", &name));
                    None
                }
                Err(_) => {
                    bot.send_plain_reply(&message, "Error querying guardian by name.".into());
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
            return bot.send_plain_reply(&message, info);
        }

        let command = args[1];
        let value = args[2];

        let mut guardian = guardian;

        use diesel_derives_traits::Model;

        match command {
            "psn" => {
                guardian.psn_name = value.into();
                guardian.save(&connection).expect("Failed to update PSN");
                bot.send_plain_reply(&message, "Updated guardian PSN".into());
            }
            "clan" => {
                let value = if value == "delete" {
                    None
                } else {
                    Some(value.into())
                };
                guardian.psn_clan = value;
                guardian.save(&connection).expect("Failed to update clan");
                bot.send_plain_reply(&message, "Updated guardian clan".into());
            }
            "email" => {
                let value = if value == "delete" {
                    None
                } else {
                    Some(value.into())
                };
                guardian.email = value;
                guardian.save(&connection).expect("Failed to update email");
                bot.send_plain_reply(&message, "Updated guardian email".into());
            }
            _ => {
                bot.send_plain_reply(&message, "Unknown information field".into());
            }
        }
    }
}
