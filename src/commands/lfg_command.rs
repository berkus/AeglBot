use {
    crate::{
        bot_actor::{BotActorMsg, CommandMsg, Format, Notify},
        commands::{match_command, validate_username},
        datetime::{format_start_time, reference_date},
        models::{Activity, ActivityShortcut, NewPlannedActivity, NewPlannedActivityMember},
        BotCommand,
    },
    chrono::prelude::*,
    chrono_english::{parse_date_string, Dialect},
    chrono_tz::Europe::Moscow,
    diesel::{self, prelude::*},
    diesel_derives_traits::{Model, NewModel},
    ractor::{cast, Actor, ActorProcessingErr},
};

command_actor!(LfgCommand, [ActorUpdateMessage]);

impl LfgCommand {
    fn send_reply<S>(
        &self,
        message: &CommandMsg,
        reply: S,
        format: Format,
    ) -> Result<(), ActorProcessingErr>
    where
        S: Into<String>,
    {
        cast!(
            self.bot_ref,
            BotActorMsg::SendMessageReply(reply.into(), message.clone(), format, Notify::Off)
        );
        Ok(())
    }

    fn usage(&self, message: &CommandMsg) -> Result<(), ActorProcessingErr> {
        self.send_reply(
            message,
            "LFG usage: /lfg <b>activity</b> YYYY-MM-DD HH:MM
For a list of activity codes: /activities
Example: /lfg kf 2018-09-10 23:00
Times are in Moscow (MSK) timezone.",
            Format::Html,
        )
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

#[async_trait::async_trait]
impl Actor for LfgCommand {
    type Msg = CommandMsg;
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
        if let (Some(_), args) = match_command(message.text(), Self::prefix(), &self.bot_name) {
            log::info!("args are {:?}", args);

            if args.is_none() {
                return self.usage(&message);
            }

            // Split args in two:
            // activity spec,
            // and timespec
            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(2, ' ').collect();

            if args.len() < 2 {
                return self.usage(&message);
            }

            let activity = args[0];
            let timespec = args[1];
            let connection = self.connection();

            log::info!("Adding activity `{}` at `{}`", &activity, &timespec);

            if let Some(guardian) = validate_username(&self.bot_ref, &message, &connection) {
                let act = ActivityShortcut::find_one_by_name(&connection, activity)
                    .expect("Failed to load Activity shortcut");

                if act.is_none() {
                    return self.send_reply(
                        &message,
                        format!(
                            "Activity {} was not found. Use /activities to see the list.",
                            activity
                        ),
                        Format::Plain,
                    );
                }
                // Parse input in MSK timezone...
                let start_time =
                    parse_date_string(timespec, Local::now().with_timezone(&Moscow), Dialect::Uk);
                // @todo Honor TELEGRAM_BOT_TIMEZONE envvar

                if start_time.is_err() {
                    return self.send_reply(
                        &message,
                        format!("Failed to parse time {}", timespec),
                        Format::Plain,
                    );
                }

                // ...then convert back to UTC.
                let start_time = start_time.unwrap().with_timezone(&Utc);

                let act = act.unwrap();

                log::info!("...parsed `{:?}`", start_time);

                let planned_activity = NewPlannedActivity {
                    author_id: guardian.id,
                    activity_id: act.link,
                    start: start_time,
                };

                use diesel::result::Error;

                connection
                    .transaction::<_, Error, _>(|| {
                        let planned_activity = planned_activity
                            .save(&connection)
                            .expect("Unexpected error saving LFG group");

                        let planned_activity_member = NewPlannedActivityMember {
                            user_id: guardian.id,
                            planned_activity_id: planned_activity.id,
                            added: reference_date(),
                        };

                        planned_activity_member
                            .save(&connection)
                            .expect("Unexpected error saving LFG group creator");

                        // @todo Duh refactor error handling here.
                        let activity = Activity::find_one(&connection, &act.link)
                            .expect("Couldn't find linked activity")
                            .unwrap();

                        self.send_reply(
                            &message,
                            format!(
                                "{guarName} is looking for {groupName} group {onTime}
{joinPrompt}
Enter `/edit{actId} details <free form description text>` to specify more details about the event.",
                                guarName = guardian,
                                groupName = activity.format_name(),
                                onTime = format_start_time(start_time, reference_date()),
                                joinPrompt = planned_activity.join_prompt(&connection),
                                actId = planned_activity.id
                            ),
                            Format::Plain,
                        )
                        .unwrap();

                        Ok(())
                    })
                    .expect("never happens, but please implement error handling");
            }
        }
        Ok(())
    }
}
