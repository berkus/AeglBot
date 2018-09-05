use crate::DbConnection;
use crate::{
    commands::{decapitalize, send_plain_reply, validate_username, BotCommand},
    datetime::{format_start_time, reference_date},
};
use diesel::{self, associations::HasTable, prelude::*};
use diesel_derives_traits::{Model, NewModel};
use futures::Future;
use models::{Activity, PlannedActivity, PlannedActivityMember};
use telebot::{functions::*, RcBot};

pub struct CancelCommand;

impl CancelCommand {
    fn usage(bot: &RcBot, message: telebot::objects::Message) {
        send_plain_reply(
            bot,
            &message,
            "To leave a fireteam provide fireteam id
Fireteam IDs are available from output of /list command."
                .into(),
        );
    }
}

impl BotCommand for CancelCommand {
    fn prefix() -> &'static str {
        "cancel"
    }

    fn description() -> &'static str {
        "Leave joined activity"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        activity_id: Option<String>,
        connection: &DbConnection,
    ) {
        if activity_id.is_none() {
            return CancelCommand::usage(bot, message);
        }

        let activity_id = activity_id.unwrap().parse::<i32>();
        if let Err(_) = activity_id {
            return CancelCommand::usage(bot, message);
        }

        let activity_id = activity_id.unwrap();

        if let Some(guardian) = validate_username(bot, &message, connection) {
            let planned =
                PlannedActivity::find_one(connection, &activity_id).expect("Failed to run SQL");

            if planned.is_none() {
                return send_plain_reply(
                    bot,
                    &message,
                    format!("Activity {} was not found.", activity_id),
                );
            }

            let planned = planned.unwrap();

            let member = planned.find_member(connection, &guardian);

            if member.is_none() {
                return send_plain_reply(bot, &message, "You are not part of this group.".into());
            }

            let member = member.unwrap();

            // @TODO if activity.too_old(2.hours) msg(cannot cancel too old)

            member.destroy(connection);

            let act_name = planned.activity(connection).format_name();
            let act_time = decapitalize(format_start_time(planned.start, reference_date()));

            let suffix = if planned.members(connection).len() == 0 {
                planned.destroy(connection);
                "This fireteam is disbanded and can no longer be joined.".into()
            } else {
                format!(
                    "{} are going
{}",
                    planned.members_formatted_list(connection),
                    planned.join_prompt(connection)
                )
            };

            send_plain_reply(
                bot,
                &message,
                format!(
                    "{guarName} has left {actName} group {actTime}
{suffix}",
                    guarName = guardian.format_name(),
                    actName = act_name,
                    actTime = act_time,
                    suffix = suffix
                ),
            );
        }
    }
}
