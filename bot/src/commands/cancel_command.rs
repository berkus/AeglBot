use {
    crate::{
        actors::bot_actor::ActorUpdateMessage,
        commands::{decapitalize, match_command, validate_username},
        render_template_or_err,
    },
    chrono::Duration,
    culpa::throws,
    entity::{plannedactivities, plannedactivitymembers},
    kameo::message::Context,
    libbot::datetime::{format_start_time, reference_date},
    sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
};

command_actor!(CancelCommand, "cancel", "Leave joined activity");

impl Message<ActorUpdateMessage> for CancelCommand {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(&mut self, message: ActorUpdateMessage, _ctx: &mut Context<Self, Self::Reply>) {
        if let (Some(_), activity_id) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if activity_id.is_none() {
                return self.usage(&message).await;
            }

            let activity_id = activity_id.unwrap().parse::<i32>();
            if activity_id.is_err() {
                return self.usage(&message).await;
            }

            let activity_id = activity_id.unwrap();

            let connection = self.connection();

            if let Some(guardian) = validate_username(&self.bot_ref, &message, connection).await {
                let planned = plannedactivities::Entity::find_by_id(activity_id)
                    .one(connection)
                    .await
                    .expect("❌ Failed to run SQL");

                if planned.is_none() {
                    return self
                        .send_reply(
                            &message,
                            format!("❌ Activity {} was not found.", activity_id),
                        )
                        .await;
                }

                let planned = planned.unwrap();

                let member = plannedactivitymembers::Entity::find()
                    .filter(plannedactivitymembers::Column::PlannedActivityId.eq(activity_id))
                    .filter(plannedactivitymembers::Column::UserId.eq(guardian.id))
                    .one(connection)
                    .await
                    .expect("❌ Failed to find member");

                if member.is_none() {
                    return self
                        .send_reply(&message, "❌ You are not a part of this group.")
                        .await;
                }

                if chrono::DateTime::<chrono::Utc>::from(planned.start)
                    < reference_date() - Duration::hours(1)
                {
                    return self
                        .send_reply(&message, "❌ You can not leave activities from the past.")
                        .await;
                }

                let member = member.unwrap();

                // Delete the member
                if plannedactivitymembers::Entity::delete_by_id(member.id)
                    .exec(connection)
                    .await
                    .is_err()
                {
                    return self
                        .send_reply(&message, "❌ Failed to remove group member".to_string())
                        .await;
                }

                let act_name = planned.activity(connection).await?.unwrap().format_name();
                let act_time = decapitalize(&format_start_time(
                    chrono::DateTime::<chrono::Utc>::from(planned.start),
                    reference_date(),
                ));

                let suffix = if planned.members_count(connection).await? == 0 {
                    if plannedactivities::Entity::delete_by_id(activity_id)
                        .exec(connection)
                        .await
                        .is_err()
                    {
                        return self
                            .send_reply(
                                &message,
                                "❌ Failed to remove planned activity".to_string(),
                            )
                            .await;
                    }
                    render_template_or_err!("cancel/disbanded")
                } else {
                    format!(
                        "{} are going\n{}",
                        planned.members_formatted_list(connection).await?,
                        planned.join_prompt(connection).await?
                    )
                };

                self.send_reply(
                    &message,
                    render_template_or_err!(
                        "cancel/left",
                        ("guardian_name" => &guardian.telegram_name),
                        ("activity_name" => &act_name),
                        ("activity_time" => &act_time),
                        ("suffix" => &suffix)
                    ),
                )
                .await;
            }
        }
    }
}
