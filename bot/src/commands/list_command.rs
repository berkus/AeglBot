use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format},
        commands::{match_command, validate_username},
        datetime::reference_date,
        render_template, BotCommand,
    },
    entity::plannedactivities,
    kameo::message::Context,
    sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
};

command_actor!(ListCommand, [ActorUpdateMessage]);

impl BotCommand for ListCommand {
    fn prefix() -> &'static str {
        "/list"
    }

    fn description() -> &'static str {
        "List current events"
    }
}

impl Message<ActorUpdateMessage> for ListCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let connection = self.connection();

        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            if let Some(_guardian) = validate_username(&self.bot_ref, &message, connection).await {
                let upcoming_events = plannedactivities::Entity::find()
                    .filter(plannedactivities::Column::Start.gt(reference_date()))
                    .all(connection)
                    .await
                    .unwrap_or_default();

                // Simplified event data for template - you may want to expand this
                #[derive(serde::Serialize)]
                struct EventTemplate {
                    id: String,
                    activity_id: String,
                    start: String,
                    details: String,
                }

                let events_data: Vec<_> = upcoming_events
                    .iter()
                    .map(|event| EventTemplate {
                        id: event.id.to_string(),
                        activity_id: event.activity_id.to_string(),
                        start: event.start.to_string(),
                        details: event.details.clone().unwrap_or_default(),
                    })
                    .collect();

                let output = render_template!("list/planned", ("events", &events_data));

                let output = if output.is_err() {
                    output.unwrap_err()
                } else {
                    output.unwrap()
                };

                self.send_reply_with_format(&message, output, Format::Html)
                    .await;
            }
        }
    }
}
