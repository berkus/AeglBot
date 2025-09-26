use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format},
        commands::{match_command, validate_username},
    },
    entity::prelude::PlannedActivities,
    kameo::message::Context,
};

command_actor!(ListCommand, "list", "List current events");

impl Message<ActorUpdateMessage> for ListCommand {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            let connection = self.connection();

            if let Some(guardian) = validate_username(&self.bot_ref, &message, connection).await {
                let events_data = PlannedActivities::upcoming_activities(connection).await;

                let output = super::render_events_list(
                    &events_data,
                    connection,
                    Some(&guardian),
                    "list/planned",
                )
                .await?;

                self.send_reply_with_format(&message, output, Format::Html)
                    .await;
            }
        }
        Ok(())
    }
}
