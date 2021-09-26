use chrono::prelude::*;
use chrono::Duration;
use chrono_english::{parse_date_string, Dialect};
use chrono_tz::Europe::Moscow;
use commands::validate_username;
use crate::{Bot, BotCommand, DbConnection};
use datetime::reference_date;
use diesel_derives_traits::Model;
use models::{ActivityShortcut, PlannedActivity};

pub struct EditCommand;

command_ctor!(EditCommand);

impl EditCommand {
    fn usage(bot: &Bot, message: &telebot::objects::Message) {
        bot.send_plain_reply(
            &message,
            "Usage:

ActivityIDs are available from output of /list command.

/edit ActivityID time <new time>
    Change activity time to a new one. Accepted time spec
    is the same as in /lfg command.

/edit ActivityID details <free form details description>
    Replaces old /details command.
    To update activity details enter text,
    to delete details use `delete` instead of text.

/edit ActivityID activity <new activity shortcut>
    Change type of activity, list of shortcuts
    is available from output of /activities command"
                .into(),
        );
    }
}

impl BotCommand for EditCommand {
    fn prefix(&self) -> &'static str {
        "/edit"
    }

    fn description(&self) -> &'static str {
        "Edit existing activity"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        args: Option<String>,
    ) {
        let connection = bot.connection();

        if args.is_none() {
            return EditCommand::usage(bot, &message);
        }
        let args = args.unwrap();

        let args: Vec<_> = args.splitn(3, ' ').collect();
        if args.len() != 3 {
            return EditCommand::usage(bot, &message);
        }

        if validate_username(bot, &message, &connection).is_some() {
            let id = args[0].parse::<i32>();
            if id.is_err() {
                return bot.send_plain_reply(&message, "ActivityID must be a number".into());
            }
            let id = id.unwrap();

            let planned = PlannedActivity::find_one(&connection, &id).expect("Failed to run SQL");

            if planned.is_none() {
                return bot.send_plain_reply(&message, format!("Activity {} was not found.", id));
            }
            let mut planned = planned.unwrap();

            if planned.start < reference_date() - Duration::hours(1) {
                return bot.send_plain_reply(&message, "You can not edit past activities.".into());
            }

            match args[1] {
                "time" => {
                    let timespec = args[2];
                    let start_time = parse_date_string(
                        timespec,
                        Local::now().with_timezone(&Moscow),
                        Dialect::Uk,
                    );
                    // @todo Honor TELEGRAM_BOT_TIMEZONE envvar

                    if start_time.is_err() {
                        return bot.send_plain_reply(
                            &message,
                            format!("Failed to parse time {}", timespec),
                        );
                    }

                    // ...then convert back to UTC.
                    let start_time = start_time.unwrap().with_timezone(&Utc);

                    log::info!("...parsed `{:?}`", start_time);

                    if planned.start < reference_date() - Duration::hours(1) {
                        return bot.send_plain_reply(
                            &message,
                            "You can not set activity time in the past.".into(),
                        );
                    }

                    planned.start = start_time;

                    if planned.save(&connection).is_err() {
                        return bot
                            .send_plain_reply(&message, "Failed to update start time.".into());
                    }

                    bot.send_plain_reply(&message, "Start time updated.".into());
                }
                "details" => {
                    let description = args[2];
                    planned.details = if description == "delete" {
                        Some(String::new())
                    } else {
                        Some(description.to_string())
                    };
                    if planned.save(&connection).is_err() {
                        return bot.send_plain_reply(&message, "Failed to update details.".into());
                    }

                    bot.send_plain_reply(&message, "Details updated.".into());
                }
                "activity" => {
                    let activity = args[2];

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
                    }

                    let act = act.unwrap();

                    planned.activity_id = act.link;

                    if planned.save(&connection).is_err() {
                        return bot.send_plain_reply(&message, "Failed to update activity.".into());
                    }

                    bot.send_plain_reply(&message, "Activity updated.".into());
                }
                x => {
                    bot.send_plain_reply(&message, format!("Unknown attribute {}", x));
                }
            }
        }
    }
}
