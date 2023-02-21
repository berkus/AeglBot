use {
    crate::{
        bot_actor::{ActorUpdateMessage, BotActorMsg, Format, Notify},
        commands::{match_command, validate_username},
        datetime::nowtz,
        models::PlannedActivity,
        BotCommand, TERA,
    },
    diesel::{self, dsl::IntervalDsl, prelude::*},
    ractor::{cast, Actor, ActorProcessingErr},
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

#[async_trait::async_trait]
impl Actor for ListCommand {
    type Msg = ActorUpdateMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        todo!()
    }

    // fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
    async fn handle(
        &self,
        myself: ActorRef<Self>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let (Some(_), _) = match_command(message.text(), Self::prefix(), &self.bot_name) {
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
                let output = "booo".into();

                cast!(
                    self.bot_ref,
                    BotActorMsg::SendMessageReply(output, message, Format::Html, Notify::Off)
                );
            }
        }
        Ok(())
    }
}
