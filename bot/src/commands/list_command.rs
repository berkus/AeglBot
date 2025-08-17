use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format},
        commands::{match_command, validate_username},
        render_template, BotCommand,
    },
    entity::prelude::PlannedActivities,
    futures::future::try_join_all,
    kameo::message::Context,
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
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let connection = self.connection();

        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            if let Some(guardian) = validate_username(&self.bot_ref, &message, connection).await {
                // let count = self.activity(connection).max_fireteam_size as usize
                //     - self.members_count(connection);

                let events_data = PlannedActivities::upcoming_activities(connection).await;
                let futures = events_data
                    .iter()
                    .map(|event| event.to_template(Some(&guardian), connection));
                let events_data = try_join_all(futures).await?;

                let output = render_template!("list/planned", ("events", &events_data));

                let output = if let Ok(item) = output {
                    item
                } else {
                    output.unwrap_err()
                };

                self.send_reply_with_format(&message, output, Format::Html)
                    .await;
            }
        }
        Ok(())
    }
}
