use {
    crate::{
        commands::{decapitalize, validate_username},
        datetime::{format_start_time, reference_date},
        models::{Activity, PlannedActivity, PlannedActivityMember},
        BotCommand, BotMenu, DbConnection,
    },
    chrono::Duration,
    diesel::{self, associations::HasTable, prelude::*},
    diesel_derives_traits::{Model, NewModel},
    futures::Future,
    teloxide::prelude::*,
};

#[derive(Clone)]
pub struct CancelCommand;

command_ctor!(CancelCommand);

impl CancelCommand {
    fn usage(bot: &BotMenu, message: &&UpdateWithCx<AutoSend<Bot>, Message>) {
        bot.send_plain_reply(
            &message,
            "To leave a fireteam provide fireteam id
Fireteam IDs are available from output of /list command."
                .into(),
        );
    }
}

impl BotCommand for CancelCommand {
    fn prefix(&self) -> &'static str {
        "/cancel"
    }

    fn description(&self) -> &'static str {
        "Leave joined activity"
    }

    fn execute(
        &self,
        bot: &BotMenu,
        message: &UpdateWithCx<AutoSend<Bot>, Message>,
        _command: Option<String>,
        activity_id: Option<String>,
    ) {
        if activity_id.is_none() {
            return CancelCommand::usage(bot, &message);
        }

        let activity_id = activity_id.unwrap().parse::<i32>();
        if activity_id.is_err() {
            return CancelCommand::usage(bot, &message);
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

            if member.is_none() {
                return bot.send_plain_reply(&message, "You are not part of this group.".into());
            }

            if planned.start < reference_date() - Duration::hours(1) {
                return bot.send_plain_reply(&message, "You can not leave past activities.".into());
            }

            let member = member.unwrap();

            if member.destroy(&connection).is_err() {
                return bot.send_plain_reply(&message, "Failed to remove group member".into());
            }

            let act_name = planned.activity(&connection).format_name();
            let act_time = decapitalize(&format_start_time(planned.start, reference_date()));

            let suffix = if planned.members(&connection).is_empty() {
                if planned.destroy(&connection).is_err() {
                    return bot
                        .send_plain_reply(&message, "Failed to remove planned activity".into());
                }
                "This fireteam is disbanded and can no longer be joined.".into()
            } else {
                format!(
                    "{} are going
{}",
                    planned.members_formatted_list(&connection),
                    planned.join_prompt(&connection)
                )
            };

            bot.send_plain_reply(
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
