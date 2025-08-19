use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format},
        commands::{match_command, validate_username},
        datetime::{format_start_time, reference_date},
        BotCommand,
    },
    entity::{activities, activityshortcuts, plannedactivities, plannedactivitymembers},
    kameo::message::Context,
    sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set},
    two_timer::parse,
};

command_actor!(LfgCommand, [ActorUpdateMessage]);

impl LfgCommand {
    async fn usage(&self, message: &ActorUpdateMessage) {
        self.send_reply_with_format(
            message,
            "LFG usage: /lfg <b>activity</b> YYYY-MM-DD HH:MM
For a list of activity codes: /activities
Example: /lfg kf 2018-09-10 23:00
Times are in Moscow (MSK) timezone.",
            Format::Html,
        )
        .await;
    }
}

impl BotCommand for LfgCommand {
    fn prefix() -> &'static str {
        "/lfg"
    }

    fn description() -> &'static str {
        "Create a new Looking For Group event"
    }
}

impl Message<ActorUpdateMessage> for LfgCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handle_message(message).await;
    }
}

impl LfgCommand {
    async fn handle_message(&self, message: ActorUpdateMessage) {
        let connection = self.connection();

        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            log::info!("args are {:?}", args);

            if args.is_none() {
                return self.usage(&message).await;
            }

            // Split args in two:
            // activity spec,
            // and timespec
            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(2, ' ').collect();

            if args.len() != 2 {
                return self.usage(&message).await;
            }

            let activity = args[0];
            let timespec = args[1];

            log::info!("Adding activity `{}` at `{}`", &activity, &timespec);

            if let Some(guardian) = validate_username(&self.bot_ref, &message, connection).await {
                let act = activityshortcuts::Entity::find()
                    .filter(activityshortcuts::Column::Name.eq(activity))
                    .one(connection)
                    .await
                    .expect("Failed to load Activity shortcut");

                if act.is_none() {
                    return self
                        .send_reply(
                            &message,
                            format!(
                                "Activity {} was not found. Use /activities to see the list.",
                                activity
                            ),
                        )
                        .await;
                }
                // Parse input in MSK timezone...
                let start_time = parse(timespec, None);
                // @todo Honor TELEGRAM_BOT_TIMEZONE envvar

                if start_time.is_err() {
                    return self
                        .send_reply(&message, format!("Failed to parse time {}", timespec))
                        .await;
                }

                // ...then convert back to UTC.
                let start_time = start_time.unwrap().0.and_utc();

                let act = act.unwrap();

                log::info!("...parsed `{:?}`", start_time);

                let planned_activity = plannedactivities::ActiveModel {
                    author_id: Set(guardian.id),
                    activity_id: Set(act.link),
                    start: Set(start_time.into()),
                    ..Default::default()
                };

                // Note: Simplified without transaction for now

                let planned_activity = planned_activity
                    .insert(connection)
                    .await
                    .expect("Unexpected error saving LFG group");

                let planned_activity_member = plannedactivitymembers::ActiveModel {
                    user_id: Set(guardian.id),
                    planned_activity_id: Set(planned_activity.id),
                    added: Set(reference_date().into()),
                    ..Default::default()
                };

                planned_activity_member
                    .insert(connection)
                    .await
                    .expect("Unexpected error saving LFG group creator");

                let activity = activities::Entity::find_by_id(act.link)
                    .one(connection)
                    .await
                    .expect("Couldn't find linked activity")
                    .unwrap();

                self.send_reply(
                    &message,
                    format!(
                        "{guarName} is looking for {groupName} group {onTime}
Enter `/edit{actId} details <free form description text>` to specify more details about the event.",
                        guarName = guardian,
                        groupName = activity.format_name(),
                        onTime = format_start_time(start_time, reference_date()),
                        actId = planned_activity.id
                    ),
                )
                .await;
            }
        }
    }
}
