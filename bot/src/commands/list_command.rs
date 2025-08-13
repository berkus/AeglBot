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
                // let count = self.activity(connection).max_fireteam_size as usize
                //     - self.members_count(connection);

                let events_data: Vec<_> = PlannedActivity::upcoming_activities(&connection)
                    .iter()
                    .map(|event| event.to_template(Some(&guardian), &connection))
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
