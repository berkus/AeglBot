use {
    crate::{
        actors::bot_actor::ActorUpdateMessage,
        commands::{match_command, validate_username},
    },
    chrono::{prelude::*, Duration},
    entity::{activityshortcuts, plannedactivities},
    kameo::message::Context,
    libbot::datetime::reference_date,
    sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set},
};

command_actor!(EditCommand, "edit", "Edit existing activity");

impl Message<ActorUpdateMessage> for EditCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if args.is_none() {
                return self.usage(&message).await;
            }
            let args = args.unwrap();

            let args: Vec<_> = args.splitn(3, ' ').collect();
            if args.len() != 3 {
                return self.usage(&message).await;
            }

            let connection = self.connection();

            if validate_username(&self.bot_ref, &message, connection)
                .await
                .is_some()
            {
                let id = args[0].parse::<i32>();
                if id.is_err() {
                    return self
                        .send_reply(&message, "❌ Activity ID must be a number")
                        .await;
                }
                let id = id.unwrap();

                let planned = plannedactivities::Entity::find_by_id(id)
                    .one(connection)
                    .await
                    .expect("❌ Failed to run SQL");

                if planned.is_none() {
                    return self
                        .send_reply(&message, format!("❌ Activity {} was not found.", id))
                        .await;
                }
                let planned = planned.unwrap();

                if planned.start < reference_date() - Duration::hours(1) {
                    return self
                        .send_reply(&message, "❌ You can not edit past activities.")
                        .await;
                }

                match args[1] {
                    "time" => {
                        let timespec = args[2];
                        let start_time = match libbot::datetime::parse_time_spec(timespec) {
                            Ok(start) => start, //.and_utc(),
                            Err(_) => {
                                return self
                                    .send_reply(
                                        &message,
                                        format!("❌ Failed to parse time {}", timespec),
                                    )
                                    .await;
                            }
                        };

                        log::info!("...parsed `{:?}`", start_time);

                        if start_time < reference_date() - Duration::hours(1) {
                            return self
                                .send_reply(
                                    &message,
                                    "❌ You can not set activity time in the past.",
                                )
                                .await;
                        }

                        let mut planned: plannedactivities::ActiveModel = planned.into();
                        let offset = start_time.offset().fix();
                        planned.start = Set(start_time.with_timezone(&offset));

                        if planned.update(connection).await.is_err() {
                            return self
                                .send_reply(&message, "❌ Failed to update start time.")
                                .await;
                        }

                        self.send_reply(&message, "✅ Start time updated.").await;
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
                            return self
                                .send_reply(&message, "❌ Failed to update details.")
                                .await;
                        }

                        self.send_reply(&message, "✅ Details updated.").await;
                    }
                    "activity" => {
                        let activity = args[2];

                        let act = activityshortcuts::Entity::find()
                            .filter(activityshortcuts::Column::Name.eq(activity))
                            .one(connection)
                            .await
                            .expect("❌ Failed to load Activity shortcut");

                        if act.is_none() {
                            return self
                                .send_reply(
                                    &message,
                                    format!(
                                        "❌ Activity {} was not found. Use /activities for a list.",
                                        activity
                                    ),
                                )
                                .await;
                        }

                        let act = act.unwrap();
                        let mut planned: plannedactivities::ActiveModel = planned.into();
                        planned.activity_id = Set(act.link);

                        if planned.update(connection).await.is_err() {
                            return self
                                .send_reply(&message, "❌ Failed to update activity type.")
                                .await;
                        }

                        self.send_reply(&message, "✅ Activity type updated.").await;
                    }
                    _ => {
                        self.usage(&message).await;
                    }
                }
            }
        }
    }
}
