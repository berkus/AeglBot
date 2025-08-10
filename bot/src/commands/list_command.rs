use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{match_command, validate_username},
        datetime::reference_date,
        render_template, BotCommand,
    },
    entity::plannedactivities,
    riker::actors::Tell,
    sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
};

command_actor!(ListCommand, [ActorUpdateMessage]);

impl ListCommand {
    fn send_reply<S>(&self, message: &ActorUpdateMessage, reply: S, format: Format)
    where
        S: Into<String>,
    {
        self.bot_ref.tell(
            SendMessageReply(reply.into(), message.clone(), format, Notify::Off),
            None,
        );
    }
}

impl BotCommand for ListCommand {
    fn prefix() -> &'static str {
        "/list"
    }

    fn description() -> &'static str {
        "List current events"
    }
}

impl Receive<ActorUpdateMessage> for ListCommand {
    type Msg = ListCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        tokio::runtime::Handle::current().block_on(async {
            self.handle_message(message).await;
        });
    }
}

impl ListCommand {
    async fn handle_message(&self, message: ActorUpdateMessage) {
        let connection = self.connection();

        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            if let Some(_guardian) = validate_username(&self.bot_ref, &message, connection).await {
                let upcoming_events = plannedactivities::Entity::find()
                    .filter(plannedactivities::Column::Start.gt(reference_date()))
                    .all(connection)
                    .await
                    .unwrap_or_default();

                // Simplified event data for template - you may want to expand this
                let events_data: Vec<_> = upcoming_events
                    .iter()
                    .map(|event| {
                        serde_json::json!({
                            "id": event.id,
                            "activity_id": event.activity_id,
                            "start": event.start.to_string(),
                            "details": event.details.as_deref().unwrap_or("")
                        })
                    })
                    .collect();

                let output = render_template!("list/planned", ("events", &events_data))
                    .expect("Rendering failed");

                self.send_reply(&message, output, Format::Html);
            }
        }
    }
}
