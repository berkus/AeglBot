use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{decapitalize, match_command, validate_username},
        datetime::{format_start_time, reference_date},
        models::{NewPlannedActivityMember, PlannedActivity},
        render_template, BotCommand,
    },
    chrono::Duration,
    diesel_derives_traits::{Model, NewModel},
    riker::actors::Tell,
};

command_actor!(JoinCommand, [ActorUpdateMessage]);

impl JoinCommand {
    fn send_reply<S>(&self, message: &ActorUpdateMessage, reply: S)
    where
        S: Into<String>,
    {
        self.bot_ref.tell(
            SendMessageReply(reply.into(), message.clone(), Format::Plain, Notify::Off),
            None,
        );
    }

    fn usage(&self, message: &ActorUpdateMessage) {
        self.send_reply(
            message,
            render_template!("join/usage").expect("Failed to render join usage template"),
        );
    }
}

impl BotCommand for JoinCommand {
    fn prefix() -> &'static str {
        "/join"
    }

    fn description() -> &'static str {
        "Join existing activity from the list"
    }
}

impl Receive<ActorUpdateMessage> for JoinCommand {
    type Msg = JoinCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), activity_id) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
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

                let member = planned.find_member(&connection, Some(&guardian));

                if member.is_some() {
                    return self.send_reply(&message, "You are already part of this group.");
                }

                if planned.is_full(&connection) {
                    return self.send_reply(&message, "This activity group is full.");
                }

                if planned.start < reference_date() - Duration::hours(1) {
                    return self.send_reply(&message, "You can not join past activities.");
                }

                let planned_activity_member = NewPlannedActivityMember {
                    user_id: guardian.id,
                    planned_activity_id: planned.id,
                    added: reference_date(),
                };

                planned_activity_member
                    .save(&connection)
                    .expect("Unexpected error saving group joiner");

                // join/joined template

                let guar_name = guardian.to_string();
                let act_name = planned.activity(&connection).format_name();
                let act_time = decapitalize(&format_start_time(planned.start, reference_date()));
                let other_guars = planned.members_formatted_list(&connection);
                let join_prompt = planned.join_prompt(&connection);

                let text = render_template!(
                    "join/joined",
                    ("guarName", &guar_name),
                    ("actName", &act_name),
                    ("actTime", &act_time),
                    ("otherGuars", &other_guars),
                    ("joinPrompt", &join_prompt)
                )
                .expect("Failed to render join joined template");

                self.send_reply(&message, text);
            }
        }
    }
}
