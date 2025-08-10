use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{match_command, validate_username},
        datetime::reference_date,
        BotCommand,
    },
    chrono::{prelude::*, Duration},
    chrono_tz::Europe::Moscow,
    entity::{activityshortcuts, plannedactivities},
    riker::actors::Tell,
    sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set},
    two_timer::parse,
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
            "Edit command help:
/edit ActivityID time <new time>
    Change scheduled time for activity. Time format examples:
    \"tomorrow at 21:00\" or \"Friday at 9 pm\" or \"21:00\"

/edit ActivityID details <new description>
    Change details/description for activity.
    Use 'delete' as description to remove details.

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
        tokio::runtime::Handle::current().block_on(async {
            self.handle_message(message).await;
        });
    }
}

impl EditCommand {
    async fn handle_message(&self, message: ActorUpdateMessage) {
        let connection = self.connection();

        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if args.is_none() {
                return self.usage(&message);
            }
            let args = args.unwrap();

            let args: Vec<_> = args.splitn(3, ' ').collect();
            if args.len() != 3 {
                return self.usage(&message);
            }

            if validate_username(&self.bot_ref, &message, connection)
                .await
                .is_some()
            {
                let id = args[0].parse::<i32>();
                if id.is_err() {
                    return self.send_reply(&message, "ActivityID must be a number");
                }
                let id = id.unwrap();

                let planned = plannedactivities::Entity::find_by_id(id)
                    .one(connection)
                    .await
                    .expect("Failed to run SQL");

                if planned.is_none() {
                    return self.send_reply(&message, format!("Activity {} was not found.", id));
                }
                let planned = planned.unwrap();

                if planned.start < reference_date() - Duration::hours(1) {
                    return self.send_reply(&message, "You can not edit past activities.");
                }

                match args[1] {
                    "time" => {
                        let timespec = args[2];
                        let now = Local::now().with_timezone(&Moscow);
                        let start_time = match parse(
                            timespec,
                            Some(two_timer::Config::new().now(now.naive_local())),
                        ) {
                            Ok((start, _end, _found)) => start.and_utc(),
                            Err(_) => {
                                return self.send_reply(
                                    &message,
                                    format!("Failed to parse time {}", timespec),
                                );
                            }
                        };

                        log::info!("...parsed `{:?}`", start_time);

                        if start_time < reference_date() - Duration::hours(1) {
                            return self.send_reply(
                                &message,
                                "You can not set activity time in the past.",
                            );
                        }

                        let mut planned: plannedactivities::ActiveModel = planned.into();
                        planned.start = Set(start_time.into());

                        if planned.update(connection).await.is_err() {
                            return self.send_reply(&message, "Failed to update start time.");
                        }

                        self.send_reply(&message, "Start time updated.");
                    }
                    "details" => {
                        let description = args[2];
                        let mut planned: plannedactivities::ActiveModel = planned.into();
                        planned.details = Set(if description == "delete" {
                            Some(String::new())
                        } else {
                            Some(description.to_string())
                        });

                        if planned.update(connection).await.is_err() {
                            return self.send_reply(&message, "Failed to update details.");
                        }

                        self.send_reply(&message, "Details updated.");
                    }
                    "activity" => {
                        let activity = args[2];

                        let act = activityshortcuts::Entity::find()
                            .filter(activityshortcuts::Column::Name.eq(activity))
                            .one(connection)
                            .await
                            .expect("Failed to load Activity shortcut");

                        if act.is_none() {
                            return self.send_reply(
                                &message,
                                format!(
                                    "Activity {} was not found. Use /activities for a list.",
                                    activity
                                ),
                            );
                        }

                        let act = act.unwrap();
                        let mut planned: plannedactivities::ActiveModel = planned.into();
                        planned.activity_id = Set(act.link);

                        if planned.update(connection).await.is_err() {
                            return self.send_reply(&message, "Failed to update activity type.");
                        }

                        self.send_reply(&message, "Activity type updated.");
                    }
                    _ => {
                        self.usage(&message);
                    }
                }
            }
        }
    }
}
