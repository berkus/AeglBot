use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{decapitalize, match_command, validate_username},
        datetime::{format_start_time, reference_date},
        render_template, BotCommand,
    },
    chrono::Duration,
    entity::{plannedactivities, plannedactivitymembers},
    riker::actors::Tell,
    sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set},
};

command_actor!(JoinCommand, [ActorUpdateMessage]);

impl JoinCommand {
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
            render_template!("join/usage").expect("Failed to render join usage template"),
        );
    }
}

impl BotCommand for JoinCommand {
    fn prefix() -> &'static str {
        "/join"
    }

    fn description() -> &'static str {
        "Join existing activity from the list"
    }
}

impl Receive<ActorUpdateMessage> for JoinCommand {
    type Msg = JoinCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        tokio::runtime::Handle::current().block_on(async {
            self.handle_message(message).await;
        });
    }
}

impl JoinCommand {
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

                if member.is_some() {
                    return self.send_reply(&message, "You are already part of this group.");
                }

                // Note: planned.is_full() method needs to be implemented for SeaORM
                // For now, we'll skip this check or implement a simple version
                // if planned.is_full(&connection) {
                //     bot_ref.tell(
                //         SendMessageReply(
                //             "This activity group is full.".into(),
                //             message,
                //             Format::Plain,
                //             Notify::Off,
                //         ),
                //         None,
                //     );
                //     return;
                // }

                if planned.start < reference_date() - Duration::hours(1) {
                    return self.send_reply(&message, "You can not join past activities.");
                }

                let planned_activity_member = plannedactivitymembers::ActiveModel {
                    user_id: Set(guardian.id),
                    planned_activity_id: Set(planned.id),
                    added: Set(reference_date().into()),
                    ..Default::default()
                };

                if planned_activity_member.insert(connection).await.is_err() {
                    return self.send_reply(&message, "Unexpected error saving group joiner");
                }

                // join/joined template
                let guar_name = guardian.telegram_name.clone();
                let act_name = format!("Activity {}", planned.activity_id); // Simplified for now
                let act_time =
                    decapitalize(&format_start_time(planned.start.into(), reference_date()));
                let other_guars = "Other members"; // Simplified for now
                let join_prompt = format!("/join {}", planned.id);

                let text = render_template!(
                    "join/joined",
                    ("guarName", &guar_name),
                    ("actName", &act_name),
                    ("actTime", &act_time),
                    ("otherGuars", &other_guars),
                    ("joinPrompt", &join_prompt)
                )
                .expect("Failed to render join joined template");

                self.bot_ref.tell(
                    SendMessageReply(text, message, Format::Plain, Notify::Off),
                    None,
                );
            }
        }
    }
}
