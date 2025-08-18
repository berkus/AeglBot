use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format},
        commands::{match_command, validate_username},
        render_template_or_err, BotCommand,
    },
    chrono::Local,
    chrono_tz::Europe::Moscow,
    entity::{activities, activityshortcuts, plannedactivities, plannedactivitymembers},
    kameo::message::Context,
    libbot::datetime::{format_start_time, reference_date},
    sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set},
};

command_actor!(LfgCommand, [ActorUpdateMessage]);

impl LfgCommand {
    async fn usage(&self, message: &ActorUpdateMessage) {
        self.send_reply_with_format(message, render_template_or_err!("lfg/usage"), Format::Html)
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
                                "❌ Activity {} was not found. Use /activities to see the list.",
                                activity
                            ),
                        )
                        .await;
                }
                let now = Local::now().with_timezone(&Moscow);
                // Parse input in MSK timezone...
                let start_time =
                    two_timer::parse(timespec, two_timer::Config::new(now).default_to_past(false));
                // @todo Honor TELEGRAM_BOT_TIMEZONE envvar

                if start_time.is_err() {
                    return self
                        .send_reply(&message, format!("❌ Failed to parse time {}", timespec))
                        .await;
                }

                // ...then convert back to UTC.
                let start_time = start_time.unwrap().0; //.and_utc();

                let act = act.unwrap();

                log::info!("...parsed `{:?}`", start_time);

                use chrono::Offset;
                let offset = start_time.offset().fix();

                let planned_activity = plannedactivities::ActiveModel {
                    author_id: Set(guardian.id),
                    activity_id: Set(act.link),
                    start: Set(start_time.with_timezone(&offset)),
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
                    render_template_or_err!(
                        "lfg/created",
                        ("guarName" => &guardian.to_string()),
                        ("groupName" => &activity.format_name()),
                        (
                            "onTime" =>
                            &format_start_time(start_time.to_utc(), reference_date())
                        ),
                        ("actId" => &planned_activity.id)
                    ),
                )
                .await;
            }
        }
    }
}
