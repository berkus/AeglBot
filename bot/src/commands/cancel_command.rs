use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{decapitalize, match_command, validate_username},
        datetime::{format_start_time, reference_date},
        BotCommand,
    },
    chrono::Duration,
    entity::{plannedactivities, plannedactivitymembers},
    riker::actors::Tell,
    sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter},
};

command_actor!(CancelCommand, [ActorUpdateMessage]);

impl CancelCommand {
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
        self.send_reply(message, "To leave a fireteam provide fireteam id\nFireteam IDs are available from output of /list command.");
    }
}

impl BotCommand for CancelCommand {
    fn prefix() -> &'static str {
        "/cancel"
    }

    fn description() -> &'static str {
        "Leave joined activity"
    }
}

impl Receive<ActorUpdateMessage> for CancelCommand {
    type Msg = CancelCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        tokio::runtime::Handle::current().block_on(async {
            self.handle_message(message).await;
        });
    }
}

impl CancelCommand {
    async fn handle_message(&self, message: ActorUpdateMessage) {
        let connection = self.connection();

        if let (Some(_), activity_id) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if activity_id.is_none() {
                return self.usage(&message);
            }

            let activity_id = activity_id.unwrap().parse::<i32>();
            if activity_id.is_err() {
                return self.usage(&message);
            }

            let activity_id = activity_id.unwrap();

            if let Some(guardian) = validate_username(&self.bot_ref, &message, connection).await {
                let planned = plannedactivities::Entity::find_by_id(activity_id)
                    .one(connection)
                    .await
                    .expect("Failed to run SQL");

                if planned.is_none() {
                    return self
                        .send_reply(&message, format!("Activity {} was not found.", activity_id));
                }

                let planned = planned.unwrap();

                let member = plannedactivitymembers::Entity::find()
                    .filter(plannedactivitymembers::Column::PlannedActivityId.eq(activity_id))
                    .filter(plannedactivitymembers::Column::UserId.eq(guardian.id))
                    .one(connection)
                    .await
                    .expect("Failed to find member");

                if member.is_none() {
                    return self
                        .send_reply(&message, "You are not part of this group.".to_string());
                }

                if chrono::DateTime::<chrono::Utc>::from(planned.start)
                    < reference_date() + Duration::hours(1)
                {
                    return self
                        .send_reply(&message, "You can not leave past activities.".to_string());
                }

                let member = member.unwrap();

                // Delete the member
                if plannedactivitymembers::Entity::delete_by_id(member.id)
                    .exec(connection)
                    .await
                    .is_err()
                {
                    return self.send_reply(&message, "Failed to remove group member".to_string());
                }

                // Get activity name - simplified for now
                let act_name = format!("Activity {}", planned.activity_id);
                let act_time = decapitalize(&format_start_time(
                    chrono::DateTime::<chrono::Utc>::from(planned.start),
                    reference_date(),
                ));

                let suffix = if remaining_members == 0 {
                    if plannedactivities::Entity::delete_by_id(activity_id)
                        .exec(connection)
                        .await
                        .is_err()
                    {
                        return self
                            .send_reply(&message, "Failed to remove planned activity".to_string());
                    }
                    "This fireteam is disbanded and can no longer be joined.".into()
                } else {
                    format!(
                        "{} members remaining\nJoin with /join{}",
                        remaining_members, activity_id
                    )
                };

                self.send_reply(
                    &message,
                    format!(
                        "{guarName} has left {actName} group {actTime}\n{suffix}",
                        guarName = guardian.telegram_name,
                        actName = act_name,
                        actTime = act_time,
                        suffix = suffix
                    ),
                );
            }
        }
    }
}
