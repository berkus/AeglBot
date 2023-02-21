use {
    crate::{
        bot_actor::{BotActor, BotActorMsg},
        datetime::{d2_reset_time, reference_date, start_at_time, start_at_weekday_time},
        services::{destiny_schedule, reminder},
        BotConnection, DbConnPool,
    },
    chrono::{Timelike, Utc},
    diesel::dsl::now,
    ractor::{cast, concurrency::Duration, Actor, ActorProcessingErr, ActorRef},
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
    bot_ref: ActorRef<BotActor>,
    lfg_chat: i64,
    connection_pool: DbConnPool,
}

impl ReminderActor {
    pub fn new(bot_ref: ActorRef<BotActor>, lfg_chat: i64, connection_pool: DbConnPool) -> Self {
        Self {
            bot_ref,
            lfg_chat,
            connection_pool,
        }
    }

    pub fn connection(&self) -> BotConnection {
        self.connection_pool.get().unwrap()
    }
}

#[async_trait::async_trait]
impl Actor for ReminderActor {
    type Msg = ReminderActorMsg;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self>,
        (): Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        // Schedule first run, the actor handler will reschedule.
        cast!(myself, ReminderActorMsg::ScheduleNextMinute);
        cast!(myself, ReminderActorMsg::ScheduleNextDay);
        cast!(myself, ReminderActorMsg::ScheduleNextWeek);
        Ok(())
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
                    ChatId(self.lfg_chat),
                );
                cast!(myself, ReminderActorMsg::ScheduleNextMinute);
            }
            ReminderActorMsg::DailyReset => {
                destiny_schedule::daily_reset(self.bot_ref.clone(), ChatId(self.lfg_chat));
                cast!(myself, ReminderActorMsg::ScheduleNextDay);
            }
            ReminderActorMsg::WeeklyReset => {
                destiny_schedule::major_weekly_reset(self.bot_ref.clone(), ChatId(self.lfg_chat));
                cast!(myself, ReminderActorMsg::ScheduleNextWeek);
            }
            ReminderActorMsg::ScheduleNextMinute => {
                let next_minute = (reference_date() + chrono::Duration::minutes(1))
                    .with_second(0)
                    .unwrap();
                let duration = next_minute - Utc::now();
                myself.send_after(duration.to_std()?, || ReminderActorMsg::Reminders);
            }
            ReminderActorMsg::ScheduleNextDay => {
                let next_day = start_at_time(reference_date(), d2_reset_time());
                let duration = next_day - Utc::now();
                myself.send_after(duration.to_std()?, || ReminderActorMsg::DailyReset);
            }
            ReminderActorMsg::ScheduleNextWeek => {
                let next_week =
                    start_at_weekday_time(reference_date(), chrono::Weekday::Tue, d2_reset_time());
                let duration = next_week - Utc::now();
                myself.send_after(duration.to_std()?, || ReminderActorMsg::WeeklyReset);
            }
        }
        Ok(())
    }
}
