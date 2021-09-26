use {
    crate::{
        commands::{decapitalize, validate_username},
        datetime::{format_start_time, reference_date},
        models::{Activity, NewPlannedActivityMember, PlannedActivity, PlannedActivityMember},
        Bot, BotCommand, DbConnection,
    },
    chrono::{Duration, Local},
    diesel::{self, associations::HasTable, prelude::*},
    diesel_derives_traits::{Model, NewModel},
    futures::Future,
};

pub struct JoinCommand;

command_ctor!(JoinCommand);

impl JoinCommand {
    fn usage(bot: &Bot, message: &telebot::objects::Message) {
        bot.send_plain_reply(
            &message,
            "To join a fireteam provide fireteam id
Fireteam IDs are available from output of /list command."
                .into(),
        );
    }
}

impl BotCommand for JoinCommand {
    fn prefix(&self) -> &'static str {
        "/join"
    }

    fn description(&self) -> &'static str {
        "Join existing activity from the list"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        activity_id: Option<String>,
    ) {
        if activity_id.is_none() {
            return JoinCommand::usage(bot, &message);
        }

        let activity_id = activity_id.unwrap().parse::<i32>();
        if activity_id.is_err() {
            return JoinCommand::usage(bot, &message);
        }

        let activity_id = activity_id.unwrap();
        let connection = bot.connection();

        if let Some(guardian) = validate_username(bot, &message, &connection) {
            let planned =
                PlannedActivity::find_one(&connection, &activity_id).expect("Failed to run SQL");

            if planned.is_none() {
                return bot.send_plain_reply(
                    &message,
                    format!("Activity {} was not found.", activity_id),
                );
            }

            let planned = planned.unwrap();

            let member = planned.find_member(&connection, &guardian);

            if member.is_some() {
                return bot
                    .send_plain_reply(&message, "You are already part of this group.".into());
            }

            if planned.is_full(&connection) {
                return bot.send_plain_reply(&message, "This activity group is full.".into());
            }

            if planned.start < reference_date() - Duration::hours(1) {
                return bot.send_plain_reply(&message, "You can not join past activities.".into());
            }

            let planned_activity_member = NewPlannedActivityMember {
                user_id: guardian.id,
                planned_activity_id: planned.id,
                added: reference_date(),
            };

            planned_activity_member
                .save(&connection)
                .expect("Unexpected error saving group joiner");

            let text = format!(
                "{guarName} has joined {actName} group {actTime}
{otherGuars} are going
{joinPrompt}",
                guarName = guardian,
                actName = planned.activity(&connection).format_name(),
                actTime = decapitalize(&format_start_time(planned.start, reference_date())),
                otherGuars = planned.members_formatted_list(&connection),
                joinPrompt = planned.join_prompt(&connection)
            );

            bot.send_plain_reply(&message, text);
        }
    }
}
