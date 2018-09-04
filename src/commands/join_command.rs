use chrono::Local;
use crate::commands::{
    decapitalize, format_start_time, send_plain_reply, validate_username, BotCommand,
};
use diesel::{self, associations::HasTable, pg::PgConnection, prelude::*};
use diesel_derives_traits::{Model, NewModel};
use futures::Future;
use models::{Activity, NewPlannedActivityMember, PlannedActivity, PlannedActivityMember};
use telebot::{functions::*, RcBot};

pub struct JoinCommand;

impl JoinCommand {
    fn usage(bot: &RcBot, message: telebot::objects::Message) {
        send_plain_reply(
            bot,
            &message,
            "To join a fireteam provide fireteam id
Fireteam IDs are available from output of /list command."
                .into(),
        );
    }
}

impl BotCommand for JoinCommand {
    fn prefix() -> &'static str {
        "join"
    }

    fn description() -> &'static str {
        "Join existing activity from the list"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        activity_id: Option<String>,
        connection: &PgConnection,
    ) {
        if activity_id.is_none() {
            return JoinCommand::usage(bot, message);
        }

        let activity_id = activity_id.unwrap().parse::<i32>();
        if let Err(_) = activity_id {
            return JoinCommand::usage(bot, message);
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

            if !member.is_none() {
                return send_plain_reply(
                    bot,
                    &message,
                    "You are already part of this group.".into(),
                );
            }

            if planned.is_full() {
                return send_plain_reply(bot, &message, "This activity group is full.".into());
            }

            // if activity.too_old() msg(cannot join too old)

            let planned_activity_member = NewPlannedActivityMember {
                user_id: guardian.id,
                planned_activity_id: planned.id,
                added: Local::now().naive_local(),
            };

            planned_activity_member
                .save(connection)
                .expect("Unexpected error saving group joiner");

            let text = format!(
                "{guarName} has joined {actName} group {actTime}
{otherGuars} are going
{joinPrompt}",
                guarName = guardian,
                actName = planned.activity(connection).format_name(),
                actTime = decapitalize(format_start_time(planned.start)),
                otherGuars = planned.members_formatted_list(),
                joinPrompt = planned.join_prompt()
            );

            send_plain_reply(bot, &message, text);
        }
    }
}
