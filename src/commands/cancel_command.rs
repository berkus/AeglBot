use {
    crate::{
        bot_actor::{ActorUpdateMessage, BotActorMsg, Format, Notify},
        commands::{decapitalize, match_command, validate_username},
        datetime::{format_start_time, reference_date},
        models::PlannedActivity,
        BotCommand,
    },
    chrono::Duration,
    diesel_derives_traits::Model,
    ractor::{cast, Actor, ActorProcessingErr},
};

command_actor!(CancelCommand, [ActorUpdateMessage]);

impl CancelCommand {
    fn send_reply<S>(
        &self,
        message: &ActorUpdateMessage,
        reply: S,
    ) -> Result<(), ActorProcessingErr>
    where
        S: Into<String>,
    {
        cast!(
            self.bot_ref,
            BotActorMsg::SendMessageReply(
                reply.into(),
                message.clone(),
                Format::Plain,
                Notify::Off
            )
        );
        Ok(())
    }

    fn usage(&self, message: &ActorUpdateMessage) -> Result<(), ActorProcessingErr> {
        self.send_reply(
            message,
            "To leave a fireteam provide fireteam id
Fireteam IDs are available from output of /list command.",
        )
    }
}

impl BotCommand for CancelCommand {
    fn prefix() -> &'static str {
        "/cancel"
    }

    fn description() -> &'static str {
        "Leave joined activity"
    }
}

#[async_trait::async_trait]
impl Actor for CancelCommand {
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
        if let (Some(_), activity_id) =
            match_command(message.text(), Self::prefix(), &self.bot_name)
        {
            if activity_id.is_none() {
                return self.usage(&message);
            }

            let activity_id = activity_id.unwrap().parse::<i32>();
            if activity_id.is_err() {
                return self.usage(&message);
            }

            let activity_id = activity_id.unwrap();
            let connection = self.connection();

            if let Some(guardian) = validate_username(&self.bot_ref, &message, &connection) {
                let planned = PlannedActivity::find_one(&connection, &activity_id)
                    .expect("Failed to run SQL");

                if planned.is_none() {
                    return self
                        .send_reply(&message, format!("Activity {} was not found.", activity_id));
                }

                let planned = planned.unwrap();

                let member = planned.find_member(&connection, &guardian);

                if member.is_none() {
                    return self.send_reply(&message, "You are not part of this group.");
                }

                if planned.start < reference_date() - Duration::hours(1) {
                    return self.send_reply(&message, "You can not leave past activities.");
                }

                let member = member.unwrap();

                if member.destroy(&connection).is_err() {
                    return self.send_reply(&message, "Failed to remove group member");
                }

                let act_name = planned.activity(&connection).format_name();
                let act_time = decapitalize(&format_start_time(planned.start, reference_date()));

                let suffix = if planned.members(&connection).is_empty() {
                    if planned.destroy(&connection).is_err() {
                        return self.send_reply(&message, "Failed to remove planned activity");
                    }
                    "This fireteam is disbanded and can no longer be joined.".into()
                } else {
                    format!(
                        "{} are going
{}",
                        planned.members_formatted_list(&connection),
                        planned.join_prompt(&connection)
                    )
                };

                self.send_reply(
                    &message,
                    format!(
                        "{guarName} has left {actName} group {actTime}
{suffix}",
                        guarName = guardian.format_name(),
                        actName = act_name,
                        actTime = act_time,
                        suffix = suffix
                    ),
                );
            }
        }
        Ok(())
    }
}
