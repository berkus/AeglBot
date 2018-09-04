use crate::commands::{send_html_reply, send_plain_reply, validate_username, BotCommand};
use diesel::{self, associations::HasTable, pg::PgConnection, prelude::*};
use diesel_derives_traits::{Model, NewModel};
use futures::Future;
use models::PlannedActivity;
use telebot::{functions::*, RcBot};

pub struct DetailsCommand;

impl DetailsCommand {
    fn usage(bot: &RcBot, message: telebot::objects::Message) {
        send_html_reply(
            bot,
            &message,
            "To update fireteam details enter /details ID freeform text
To delete details use `/details ID del`.
Fireteam IDs are available from output of /list command."
                .into(),
        );
    }
}

impl BotCommand for DetailsCommand {
    fn prefix() -> &'static str {
        "details"
    }

    fn description() -> &'static str {
        "Set group details as freeform text"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        args: Option<String>,
        connection: &PgConnection,
    ) {
        info!("args are {:?}", args);

        if args.is_none() {
            return DetailsCommand::usage(bot, message);
        }

        // Split args in two:
        // activity spec,
        // and description text
        let args = args.unwrap();
        let args: Vec<&str> = args.splitn(2, ' ').collect();

        if args.len() < 2 {
            return DetailsCommand::usage(bot, message);
        }

        let activity = args[0];
        let description = args[1];

        info!("Activity `{}`, description `{}`", activity, description);

        let activity_id = activity.parse::<i32>();
        if let Err(_) = activity_id {
            return DetailsCommand::usage(bot, message);
        }

        let activity_id = activity_id.unwrap();

        if let Some(_) = validate_username(bot, &message, connection) {
            let planned =
                PlannedActivity::find_one(connection, &activity_id).expect("Failed to run SQL");

            if planned.is_none() {
                return send_plain_reply(
                    bot,
                    &message,
                    format!("Activity {} was not found.", activity_id),
                );
            }

            let mut planned = planned.unwrap();

            planned.details = if description == "del" {
                Some(String::new())
            } else {
                Some(description.to_string())
            };
            planned.save(connection);

            send_plain_reply(bot, &message, "Details updated.".into());
        }
    }
}
