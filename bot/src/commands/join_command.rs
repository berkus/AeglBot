use {
    crate::{
        actors::bot_actor::ActorUpdateMessage,
        commands::{decapitalize, match_command, validate_username},
        render_template_or_err,
    },
    chrono::Duration,
    culpa::throws,
    entity::{plannedactivitymembers, prelude::*},
    kameo::message::Context,
    libbot::datetime::{format_start_time, reference_date},
    sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set},
};

command_actor!(JoinCommand, "/join", "Join existing activity from the list");

impl JoinCommand {
    async fn join_usage(&self, message: &ActorUpdateMessage) {
        self.send_reply(message, render_template_or_err!("join/usage"))
            .await;
    }
}

impl Message<ActorUpdateMessage> for JoinCommand {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(&mut self, message: ActorUpdateMessage, _ctx: &mut Context<Self, Self::Reply>) {
        let connection = self.connection();

        if let (Some(_), activity_id) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if activity_id.is_none() {
                return self.join_usage(&message).await;
            }

            let activity_id = activity_id.unwrap().parse::<i32>();
            if activity_id.is_err() {
                return self.join_usage(&message).await;
            }

            let activity_id = activity_id.unwrap();

            if let Some(guardian) = validate_username(&self.bot_ref, &message, connection).await {
                let found = PlannedActivities::find_by_id(activity_id)
                    .find_also_related(Activities)
                    .one(connection)
                    .await?;

                if found.is_none() {
                    return self
                        .send_reply(
                            &message,
                            format!("❌ Activity {} was not found.", activity_id),
                        )
                        .await;
                }

                let (planned, activity) = found.unwrap();

                let member = plannedactivitymembers::Entity::find()
                    .filter(plannedactivitymembers::Column::PlannedActivityId.eq(activity_id))
                    .filter(plannedactivitymembers::Column::UserId.eq(guardian.id))
                    .one(connection)
                    .await
                    .expect("Failed to find member");

                if member.is_some() {
                    return self
                        .send_reply(&message, "✅ You are already part of this group.")
                        .await;
                }

                if planned.is_full(connection).await? {
                    return self
                        .send_reply(&message, "❌ This activity group is full.")
                        .await;
                }

                if planned.start < reference_date() - Duration::hours(1) {
                    return self
                        .send_reply(&message, "❌ You can not join past activities.")
                        .await;
                }

                let planned_activity_member = plannedactivitymembers::ActiveModel {
                    user_id: Set(guardian.id),
                    planned_activity_id: Set(planned.id),
                    added: Set(reference_date().into()),
                    ..Default::default()
                };

                if planned_activity_member.insert(connection).await.is_err() {
                    return self
                        .send_reply(&message, "🐛 Unexpected error saving group joiner")
                        .await;
                }

                // join/joined template - TODO: format new member correctly (with icon etc)
                let guar_name = guardian.to_string();
                let act_name = activity.expect("REASONS").format_name();
                let act_time =
                    decapitalize(&format_start_time(planned.start.into(), reference_date()));
                let other_guars = planned.members_formatted_list(connection).await?;
                let join_prompt = planned.join_prompt(connection).await?;

                let text = render_template_or_err!(
                    "join/joined",
                    ("guardian" => &guar_name),
                    ("activity_name" => &act_name),
                    ("activity_time" => &act_time),
                    ("other_guardians" => &other_guars),
                    ("join_prompt" => &join_prompt)
                );

                self.send_reply(&message, text).await;
            }
        }
    }
}
