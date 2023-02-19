use {
    crate::{
        bot_actor::BotActorMsg,
        datetime::{d2_reset_time, reference_date, start_at_time, start_at_weekday_time},
        services::{destiny_schedule, reminder},
        BotConnection, DbConnPool,
    },
    chrono::Timelike,
    ractor::{cast, Actor, ActorProcessingErr, ActorRef},
    teloxide::types::ChatId,
};

pub enum ReminderActorMsg {
    Reminders,
    DailyReset,
    WeeklyReset,
    ScheduleNextMinute,
    ScheduleNextDay,
    ScheduleNextWeek,
}

pub struct ReminderActor {
    bot_ref: ActorRef<BotActorMsg>,
    lfg_chat: i64,
    connection_pool: DbConnPool,
}

impl ReminderActor {
    pub fn connection(&self) -> BotConnection {
        self.connection_pool.get().unwrap()
    }
}

#[async_trait::async_trait]
impl Actor for ReminderActor {
    type Msg = ReminderActorMsg;
    type State = ();
    type Arguments = (ActorRef<BotActorMsg>, i64, DbConnPool);

    async fn pre_start(
        &self,
        myself: ActorRef<Self>,
        (bot_ref, lfg_chat, connection_pool): (ActorRef<BotActorMsg>, i64, DbConnPool),
    ) -> Result<Self::State, ActorProcessingErr> {
        // Schedule first run, the actor handler will reschedule.
        cast!(myself, ReminderActorMsg::ScheduleNextMinute);
        cast!(myself, ReminderActorMsg::ScheduleNextDay);
        cast!(myself, ReminderActorMsg::ScheduleNextWeek);
        // Create the initial state. -- @todo is this the state though? it's immutable stuff
        Ok((bot_ref, lfg_chat, connection_pool))
    }

    async fn handle(
        &self,
        myself: ActorRef<Self>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ReminderActorMsg::Reminders => {
                reminder::check(
                    self.bot_ref.clone(),
                    self.connection(),
                    ChatId::Id(self.lfg_chat),
                );
                cast!(myself, ReminderActorMsg::ScheduleNextMinute);
            }
            ReminderActorMsg::DailyReset => {
                destiny_schedule::daily_reset(self.bot_ref.clone(), ChatId::Id(self.lfg_chat));
                cast!(myself, ReminderActorMsg::ScheduleNextDay);
            }
            ReminderActorMsg::WeeklyReset => {
                destiny_schedule::major_weekly_reset(
                    self.bot_ref.clone(),
                    ChatId::Id(self.lfg_chat),
                );
                cast!(myself, ReminderActorMsg::ScheduleNextWeek);
            }
            ReminderActorMsg::ScheduleNextMinute => {
                ctx.schedule_at_time(
                    (reference_date() + chrono::Duration::minutes(1))
                        .with_second(0)
                        .unwrap(),
                    ctx.myself(),
                    None,
                    ReminderActorMsg::Reminders,
                );
            }
            ReminderActorMsg::ScheduleNextDay => {
                ctx.schedule_at_time(
                    start_at_time(reference_date(), d2_reset_time()),
                    ctx.myself(),
                    None,
                    ReminderActorMsg::DailyReset,
                );
            }
            ReminderActorMsg::ScheduleNextWeek => {
                ctx.schedule_at_time(
                    start_at_weekday_time(reference_date(), chrono::Weekday::Tue, d2_reset_time()),
                    ctx.myself(),
                    None,
                    ReminderActorMsg::WeeklyReset,
                );
            }
        }
        Ok(())
    }
}
