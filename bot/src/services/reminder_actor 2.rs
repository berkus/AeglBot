use {
    crate::{
        datetime::{d2_reset_time, reference_date, start_at_time, start_at_weekday_time},
        services::{destiny_schedule, reminder},
        BotConnection,
    },
    chrono::Timelike,
    kameo::{actor::ActorRef, error::Infallible, message::*, Actor},
    teloxide::types::ChatId,
};

#[derive(Clone)]
pub struct ReminderActor {
    bot_ref: ActorRef<crate::bot_actor::BotActor>,
    lfg_chat: i64,
    connection_pool: BotConnection,
}

impl ReminderActor {
    pub fn new(
        bot_ref: ActorRef<crate::bot_actor::BotActor>,
        lfg_chat: i64,
        connection_pool: BotConnection,
    ) -> Self {
        Self {
            bot_ref,
            lfg_chat,
            connection_pool,
        }
    }

    fn connection(&self) -> &BotConnection {
        &self.connection_pool
    }
}

impl Actor for ReminderActor {
    type Args = Self;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(args)
    }
}

#[derive(Clone, Debug)]
pub struct Reminders;

#[derive(Clone, Debug)]
pub struct DailyReset;

#[derive(Clone, Debug)]
pub struct WeeklyReset;

impl Message<Reminders> for ReminderActor {
    type Reply = ();
    async fn handle(
        &mut self,
        _msg: Reminders,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let bot_ref = self.bot_ref.clone();
        let connection = self.connection().clone();
        let lfg_chat = self.lfg_chat;
        let actor_ref = ctx.actor_ref().clone();

        tokio::spawn(async move {
            reminder::check(bot_ref, connection, ChatId(lfg_chat)).await;
            let _ = actor_ref.tell(ScheduleNextMinute).await;
        });
    }
}

impl Message<DailyReset> for ReminderActor {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        _msg: DailyReset,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        destiny_schedule::daily_reset(self.bot_ref.clone(), ChatId(self.lfg_chat)).await?;
        ctx.actor_ref().tell(ScheduleNextDay).await?;
        Ok(())
    }
}

impl Message<WeeklyReset> for ReminderActor {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        _msg: WeeklyReset,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        destiny_schedule::major_weekly_reset(self.bot_ref.clone(), ChatId(self.lfg_chat)).await?;
        ctx.actor_ref().tell(ScheduleNextWeek).await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ScheduleNextMinute;

#[derive(Clone, Debug)]
pub struct ScheduleNextDay;

#[derive(Clone, Debug)]
pub struct ScheduleNextWeek;

impl Message<ScheduleNextMinute> for ReminderActor {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: ScheduleNextMinute,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let target_time = (reference_date() + chrono::Duration::minutes(1))
            .with_second(0)
            .unwrap();
        let actor_ref = ctx.actor_ref().clone();

        let now = std::time::SystemTime::now();
        let target_system_time =
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(target_time.timestamp() as u64);
        if let Ok(duration) = target_system_time.duration_since(now) {
            tokio::time::sleep(duration).await;
            let _ = actor_ref.tell(Reminders).await;
        }
    }
}

impl Message<ScheduleNextDay> for ReminderActor {
    type Reply = ();
    async fn handle(
        &mut self,
        _msg: ScheduleNextDay,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let target_time = start_at_time(reference_date(), d2_reset_time());
        let actor_ref = ctx.actor_ref().clone();

        let now = std::time::SystemTime::now();
        let target_system_time =
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(target_time.timestamp() as u64);
        if let Ok(duration) = target_system_time.duration_since(now) {
            tokio::time::sleep(duration).await;
            let _ = actor_ref.tell(DailyReset).await;
        }
    }
}

impl Message<ScheduleNextWeek> for ReminderActor {
    type Reply = ();
    async fn handle(
        &mut self,
        _msg: ScheduleNextWeek,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let target_time =
            start_at_weekday_time(reference_date(), chrono::Weekday::Tue, d2_reset_time());
        let actor_ref = ctx.actor_ref().clone();

        let now = std::time::SystemTime::now();
        let target_system_time =
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(target_time.timestamp() as u64);
        if let Ok(duration) = target_system_time.duration_since(now) {
            tokio::time::sleep(duration).await;
            let _ = actor_ref.tell(WeeklyReset).await;
        }
    }
}
