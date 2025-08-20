use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format},
        commands::{match_command, validate_username},
        render_template_or_err,
    },
    entity::prelude::PlannedActivities,
    futures::future::try_join_all,
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
                let futures = events_data
                    .iter()
                    .map(|event| event.to_template(connection, Some(&guardian)));
                let events_data = try_join_all(futures).await?;

                let output = render_template_or_err!("list/planned", ("events" => &events_data));

                self.send_reply_with_format(&message, output, Format::Html)
                    .await;
            }
        }
        Ok(())
    }
}
