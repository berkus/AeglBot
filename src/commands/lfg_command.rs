use chrono::prelude::*;
use chrono_english::{parse_date_string, Dialect};
use chrono_tz::Europe::Moscow;
use crate::DbConnection;
use crate::{
    commands::{bot_command::BotCommand, send_html_reply, send_plain_reply, validate_username},
    datetime::{format_start_time, msk_to_naive, naive_to_msk, reference_date},
};
use diesel::{self, associations::HasTable, prelude::*};
use diesel_derives_traits::{Model, NewModel};
use futures::Future;
use models::{Activity, ActivityShortcut, NewPlannedActivity, NewPlannedActivityMember};
use telebot::{functions::*, RcBot};

pub struct LfgCommand;

impl LfgCommand {
    fn usage(bot: &RcBot, message: telebot::objects::Message) {
        send_html_reply(
            bot,
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
        connection: &DbConnection,
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
            let act = ActivityShortcut::find_one_by_name(connection, activity)
                .expect("Failed to load Activity shortcut");

            if act.is_none() {
                send_plain_reply(
                    bot,
                    &message,
                    format!(
                        "Activity {} was not found. Use /activities for a list.",
                        activity
                    ),
                );
            } else {
                let start_time = parse_date_string(timespec, Local::now(), Dialect::Uk);

                if let Err(_) = start_time {
                    return send_plain_reply(
                        bot,
                        &message,
                        format!("Failed to parse time {}", timespec),
                    );
                }

                let msk_time = naive_to_msk(start_time.unwrap().naive_local());
                let start_time = msk_to_naive(msk_time);
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

                connection.transaction::<_, Error, _>(|| {
                    let planned_activity = planned_activity
                        .save(connection)
                        .expect("Unexpected error saving LFG group");

                    let planned_activity_member = NewPlannedActivityMember {
                        user_id: guardian.id,
                        planned_activity_id: planned_activity.id,
                        added: Local::now().naive_local(), // @todo MSK!!
                    };

                    planned_activity_member
                        .save(connection)
                        .expect("Unexpected error saving LFG group creator");

                    let activity = Activity::find_one(connection, &act.link)
                        .expect("Couldn't find linked activity")
                        .unwrap();

                    send_plain_reply(
                        bot,
                        &message,
                        format!(
                            "{guarName} is looking for {groupName} group {onTime}
{joinPrompt}
Enter `/details {actId} free form description text` to specify more details about the event.",
                            guarName = guardian,
                            groupName = activity.format_name(),
                            onTime = format_start_time(start_time, reference_date()),
                            joinPrompt = planned_activity.join_prompt(connection),
                            actId = planned_activity.id
                        ),
                    );

                    Ok(())
                });
            }
        }
    }
}
