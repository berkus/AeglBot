use chrono::prelude::*;
use chrono_english::{parse_date_string, Dialect};
use crate::commands::{
    bot_command::BotCommand, format_start_time, send_html_reply, send_plain_reply,
    validate_username,
};
use diesel::{self, pg::PgConnection, prelude::*};
use futures::Future;
use models::{ActivityShortcut, NewPlannedActivity, NewPlannedActivityMember};
use telebot::{functions::*, RcBot};

pub struct LfgCommand;

impl LfgCommand {
    fn usage(bot: &RcBot, message: telebot::objects::Message) {
        send_html_reply(
            bot,
            message,
            "LFG usage: /lfg <b>activity</b> timespec
For a list of activity codes: /activities
Example: /lfg kf tomorrow 23:00
(NB: times are in MSK timezone by default)"
                .into(),
        );
    }
}

impl BotCommand for LfgCommand {
    fn prefix() -> &'static str {
        "lfg"
    }

    fn description() -> &'static str {
        "Create a new Looking For Group event"
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
            return LfgCommand::usage(bot, message);
        }

        // Split args in two:
        // activity spec,
        // and timespec
        let args = args.unwrap();
        let args: Vec<&str> = args.splitn(2, ' ').collect();

        if args.len() < 2 {
            return LfgCommand::usage(bot, message);
        }

        let activity = args[0];
        let timespec = args[1];

        info!("Adding activity `{}` at `{}`", &activity, &timespec);

        if let Some(guardian) = validate_username(bot, &message, connection) {
            use schema::activityshortcuts::dsl::*;

            let act = activityshortcuts
                .filter(name.eq(activity))
                .optional::<ActivityShortcut>(&connection)
                .expect("Failed to load Activity shortcut");

            if act.is_none() {
                send_plain_reply(
                    bot,
                    message,
                    format!(
                        "Activity {} was not found. Use /activities for a list.",
                        activity
                    ),
                );
            } else {
                let start_time = parse_date_string(timespec, Local::now(), Dialect::Us);

                info!("...parsed `{:?}`", start_time);

                let planned_activity = NewPlannedActivity {
                    author: guardian.id,
                    activity: act.link,
                    start: start_time,
                };

                use schema::plannedactivities::dsl::*;
                use schema::plannedactivitymembers::dsl::*;

                diesel::insert_into(plannedactivities::table)
                    .values(&planned_activity)
                    .execute(connection)
                    .expect("Unexpected error saving LFG group");

                let planned_activity_member = NewPlannedActivityMember {
                    user: guardian.id,
                    activity: planned_activity.id,
                };

                diesel::insert_into(plannedactivitymembers::table)
                    .values(&planned_activity_member)
                    .execute(connection)
                    .expect("Unexpected error saving LFG group creator");

                // Todo: always post to lfg chat?
                send_plain_reply(
                    bot,
                    message,
                    format!(
                        "{guarName} is looking for {groupName} group {onTime}
{joinPrompt}
Enter `/details {actId} free form description text` to specify more details about the event.",
                        guarName = guardian,
                        groupName = act.link.format_name(),
                        onTime = format_start_time(start_time),
                        joinPrompt = planned_activity.join_prompt(),
                        actId = planned_activity.id
                    ),
                );
            }
        }
    }
}
