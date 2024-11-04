use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{match_command, validate_username},
        models::{ActivityShortcut, PlannedActivity},
        BotCommand,
    },
    chrono::{prelude::*, Duration},
    chrono_english::{parse_date_string, Dialect},
    chrono_tz::Europe::Moscow,
    diesel_derives_traits::Model,
    riker::actors::Tell,
    libbot::datetime::reference_date,
};

command_actor!(EditCommand, [ActorUpdateMessage]);

impl EditCommand {
    fn send_reply<S>(&self, message: &ActorUpdateMessage, reply: S)
    where
        S: Into<String>,
    {
        self.bot_ref.tell(
            SendMessageReply(reply.into(), message.clone(), Format::Plain, Notify::Off),
            None,
        );
    }

    fn usage(&self, message: &ActorUpdateMessage) {
        self.send_reply(
            message,
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
    is available from output of /activities command",
        );
    }
}

impl BotCommand for EditCommand {
    fn prefix() -> &'static str {
        "/edit"
    }

    fn description() -> &'static str {
        "Edit existing activity"
    }
}

impl Receive<ActorUpdateMessage> for EditCommand {
    type Msg = EditCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            let connection = self.connection();

            if args.is_none() {
                return self.usage(&message);
            }
            let args = args.unwrap();

            let args: Vec<_> = args.splitn(3, ' ').collect();
            if args.len() != 3 {
                return self.usage(&message);
            }

            if validate_username(&self.bot_ref, &message, &connection).is_some() {
                let id = args[0].parse::<i32>();
                if id.is_err() {
                    return self.send_reply(&message, "ActivityID must be a number");
                }
                let id = id.unwrap();

                let planned =
                    PlannedActivity::find_one(&connection, &id).expect("Failed to run SQL");

                if planned.is_none() {
                    return self.send_reply(&message, format!("Activity {} was not found.", id));
                }
                let mut planned = planned.unwrap();

                if planned.start < reference_date() - Duration::hours(1) {
                    return self.send_reply(&message, "You can not edit past activities.");
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
                            return self.send_reply(
                                &message,
                                format!("Failed to parse time {}", timespec),
                            );
                        }

                        // ...then convert back to UTC.
                        let start_time = start_time.unwrap().with_timezone(&Utc);

                        log::info!("...parsed `{:?}`", start_time);

                        if planned.start < reference_date() - Duration::hours(1) {
                            return self.send_reply(
                                &message,
                                "You can not set activity time in the past.",
                            );
                        }

                        planned.start = start_time;

                        if planned.save(&connection).is_err() {
                            return self.send_reply(&message, "Failed to update start time.");
                        }

                        self.send_reply(&message, "Start time updated.");
                    }
                    "details" => {
                        let description = args[2];
                        planned.details = if description == "delete" {
                            Some(String::new())
                        } else {
                            Some(description.to_string())
                        };
                        if planned.save(&connection).is_err() {
                            return self.send_reply(&message, "Failed to update details.");
                        }

                        self.send_reply(&message, "Details updated.");
                    }
                    "activity" => {
                        let activity = args[2];

                        let act = ActivityShortcut::find_one_by_name(&connection, activity)
                            .expect("Failed to load Activity shortcut");

                        if act.is_none() {
                            self.send_reply(
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
                            return self.send_reply(&message, "Failed to update activity.");
                        }

                        self.send_reply(&message, "Activity updated.");
                    }
                    x => {
                        self.send_reply(&message, format!("Unknown attribute {}", x));
                    }
                }
            }
        }
    }
}
