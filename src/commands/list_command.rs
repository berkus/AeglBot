use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{match_command, validate_username},
        models::PlannedActivity,
        render_template, BotCommand,
    },
    riker::actors::Tell,
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
        if let (Some(_), _) = match_command(message.update.text(), Self::prefix(), &self.bot_name) {
            let connection = self.connection();

            if let Some(guardian) = validate_username(&self.bot_ref, &message, &connection) {
                // let count = self.activity(connection).max_fireteam_size as usize
                //     - self.members_count(connection);
                let upcoming_events: Vec<_> = PlannedActivity::upcoming_activities(&connection)
                    .iter()
                    .map(|s| s.to_template(Some(&guardian), &connection))
                    .collect();

                let output = render_template!("list/planned", ("events", &upcoming_events))
                    .expect("Rendering failed");

                self.send_reply(&message, output, Format::Html);
            }
        }
    }
}
