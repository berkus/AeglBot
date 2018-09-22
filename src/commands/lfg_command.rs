use chrono::prelude::*;
use chrono_english::{parse_date_string, Dialect};
use chrono_tz::Europe::Moscow;
use crate::{
    commands::validate_username,
    datetime::{format_start_time, reference_date, BotDateTime},
};
use crate::{Bot, BotCommand, DbConnection};
use diesel::{self, associations::HasTable, prelude::*};
use diesel_derives_traits::{Model, NewModel};
use futures::Future;
use models::{Activity, ActivityShortcut, NewPlannedActivity, NewPlannedActivityMember};

pub struct LfgCommand;

command_ctor!(LfgCommand);

impl LfgCommand {
    fn usage(bot: &Bot, message: &telebot::objects::Message) {
        bot.send_html_reply(
            &message,
            "LFG usage: /lfg <b>activity</b> YYYY-MM-DD HH:MM
For a list of activity codes: /activities
Example: /lfg kf 2018-09-10 23:00
Times are in Moscow (MSK) timezone."
                .into(),
        );
    }
}

impl BotCommand for LfgCommand {
    fn prefix(&self) -> &'static str {
        "/lfg"
    }

    fn description(&self) -> &'static str {
        "Create a new Looking For Group event"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        args: Option<String>,
    ) {
        info!("args are {:?}", args);

        if args.is_none() {
            return LfgCommand::usage(bot, &message);
        }

        // Split args in two:
        // activity spec,
        // and timespec
        let args = args.unwrap();
        let args: Vec<&str> = args.splitn(2, ' ').collect();

        if args.len() < 2 {
            return LfgCommand::usage(bot, &message);
        }

        let activity = args[0];
        let timespec = args[1];
        let connection = bot.connection();

        info!("Adding activity `{}` at `{}`", &activity, &timespec);

        if let Some(guardian) = validate_username(bot, &message, &connection) {
            let act = ActivityShortcut::find_one_by_name(&connection, activity)
                .expect("Failed to load Activity shortcut");

            if act.is_none() {
                bot.send_plain_reply(
                    &message,
                    format!(
                        "Activity {} was not found. Use /activities for a list.",
                        activity
                    ),
                );
            } else {
                // Parse input in MSK timezone...
                let start_time =
                    parse_date_string(timespec, Local::now().with_timezone(&Moscow), Dialect::Uk);
                // @todo Honor TELEGRAM_BOT_TIMEZONE envvar

                if start_time.is_err() {
                    return bot
                        .send_plain_reply(&message, format!("Failed to parse time {}", timespec));
                }

                // ...then convert back to UTC.
                let start_time = start_time.unwrap().with_timezone(&Utc);

                let act = act.unwrap();

                info!("...parsed `{:?}`", start_time);

                let planned_activity = NewPlannedActivity {
                    author_id: guardian.id,
                    activity_id: act.link,
                    start: start_time,
                };

                use diesel::result::Error;
                use schema::plannedactivities::dsl::*;
                use schema::plannedactivitymembers::dsl::*;

                connection
                    .transaction::<_, Error, _>(|| {
                        let planned_activity = planned_activity
                            .save(&connection)
                            .expect("Unexpected error saving LFG group");

                        let planned_activity_member = NewPlannedActivityMember {
                            user_id: guardian.id,
                            planned_activity_id: planned_activity.id,
                            added: reference_date(),
                        };

                        planned_activity_member
                            .save(&connection)
                            .expect("Unexpected error saving LFG group creator");

                        let activity = Activity::find_one(&connection, &act.link)
                            .expect("Couldn't find linked activity")
                            .unwrap();

                        bot.send_plain_reply(
                            &message,
                            format!(
                                "{guarName} is looking for {groupName} group {onTime}
{joinPrompt}
Enter `/details {actId} free form description text` to specify more details about the event.",
                                guarName = guardian,
                                groupName = activity.format_name(),
                                onTime = format_start_time(start_time, reference_date()),
                                joinPrompt = planned_activity.join_prompt(&connection),
                                actId = planned_activity.id
                            ),
                        );

                        Ok(())
                    }).expect("never happens, but please implement error handling");
            }
        }
    }
}
