use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{match_command, validate_username},
        datetime::nowtz,
        models::PlannedActivity,
        BotCommand,
    },
    diesel::{self, dsl::IntervalDsl, prelude::*},
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
    fn prefix(&self) -> &'static str {
        "/list"
    }

    fn description(&self) -> &'static str {
        "List current events"
    }
}

impl Receive<ActorUpdateMessage> for ListCommand {
    type Msg = ListCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), _) = match_command(message.update.text(), self.prefix(), &self.bot_name) {
            use crate::schema::plannedactivities::dsl::*;

            let connection = self.connection();

            let upcoming_events = plannedactivities
                .filter(start.ge(nowtz() - 60_i32.minutes()))
                .order(start.asc())
                .load::<PlannedActivity>(&connection)
                .expect("TEMP loading @FIXME");

            if upcoming_events.is_empty() {
                return self.send_reply(
                    &message,
                    "No activities planned, add something with /lfg",
                    Format::Plain,
                );
            }

            if let Some(guardian) = validate_username(&self.bot_ref, &message, &connection) {
                let text = upcoming_events.iter().fold(
                    "Planned activities:\n\n".to_owned(),
                    |acc, event| {
                        acc + &format!("{}\n\n", event.display(&connection, Some(&guardian)))
                    },
                );

                self.send_reply(&message, text, Format::Html);
            }
        }
    }
}
