use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{match_command, validate_username},
        datetime::nowtz,
        models::PlannedActivity,
        BotCommand, TERA,
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

            if let Some(_guardian) = validate_username(&self.bot_ref, &message, &connection) {
                // let count = self.activity(connection).max_fireteam_size as usize
                //     - self.members_count(connection);

                use crate::schema::plannedactivities::dsl::*;
                let upcoming_events = plannedactivities
                    .filter(start.ge(nowtz() - 60_i32.minutes()))
                    .order(start.asc())
                    .load::<PlannedActivity>(&connection)
                    .expect("TEMP loading @FIXME");
                // .iter()
                // .map(|s| s.to_template(connection, _guardian));

                // let mut cx = tera::Context::new();
                // cx.insert("events", &upcoming_events);
                // let output = TERA.render("activity_list", &cx).expect("to render nicely");
                //
                //  self.send_reply(
                //     &message,
                //     &output,
                //     Format::Html,
                // );
            }
        }
    }
}
