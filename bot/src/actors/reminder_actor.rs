use {
    crate::{
        actors::bot_actor::{Format, Notify, SendMessage},
        BotConnection,
    },
    chrono::Timelike,
    culpa::throws,
    entity::prelude::*,
    kameo::{actor::ActorRef, error::Infallible, message::*, Actor},
    libbot::{
        datetime::{d2_reset_time, reference_date, start_at_time, start_at_weekday_time},
        services::destiny_schedule::{this_week_in_d1, this_week_in_d2},
    },
    teloxide::types::ChatId,
};

#[derive(Clone)]
pub struct ReminderActor {
    bot_ref: ActorRef<crate::actors::bot_actor::BotActor>,
    lfg_chat: i64,
    connection_pool: BotConnection,
}

impl Actor for ReminderActor {
    type Args = Self;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(args)
    }
}

impl ReminderActor {
    pub fn new(
        bot_ref: ActorRef<crate::actors::bot_actor::BotActor>,
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
        let connection = self.connection();
        let lfg_chat = self.lfg_chat;

        let found = PlannedActivities::upcoming_activities_alert(connection).await;

        if let Some(upcoming_events) = found {
            // @Todo: this text should be populated in tera template in `bot`
            let text = upcoming_events
                .into_iter()
                .fold("Activities starting soon:\n\n".to_owned(), |acc, event| {
                    acc + &format!("Activity {} starting soon\n\n", event.id)
                });

            let _ = bot_ref
                .tell(SendMessage(
                    text,
                    ChatId(lfg_chat),
                    Format::Html,
                    Notify::On,
                ))
                .await;
        }

        let _ = ctx.actor_ref().tell(ScheduleNextMinute).await;
    }
}

// 1. Daily resets at 20:00 MSK (17:00 UTC) every day
#[throws(kameo::error::SendError<crate::actors::bot_actor::SendMessage>)]
pub async fn daily_reset(bot: ActorRef<crate::actors::bot_actor::BotActor>, lfg_chat: ChatId) {
    bot.tell(SendMessage(
        "⚡️ Daily reset".into(),
        lfg_chat,
        Format::Plain,
        Notify::Off,
    ))
    .await?;
}

impl Message<DailyReset> for ReminderActor {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        _msg: DailyReset,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        daily_reset(self.bot_ref.clone(), ChatId(self.lfg_chat)).await?;
        ctx.actor_ref().tell(ScheduleNextDay).await?;
        Ok(())
    }
}

// 2. Weekly (main) resets at 20:00 msk every Tue
// 6. On main reset: change in Dreaming City curse
//    dreaming city on 3-week schedule
// 7. On main reset: change in Dreaming City Ascendant Challenges
//    dreaming city challenges on 6-week schedule
#[throws(kameo::error::SendError<crate::actors::bot_actor::SendMessage>)]
pub async fn major_weekly_reset(
    bot: ActorRef<crate::actors::bot_actor::BotActor>,
    lfg_chat: ChatId,
) {
    let msg = format!(
        "⚡️ Weekly reset:\n\n{d1week}\n\n{d2week}",
        d1week = this_week_in_d1(),
        d2week = this_week_in_d2(),
    );
    bot.tell(SendMessage(msg, lfg_chat, Format::Markdown, Notify::Off))
        .await?;
}

impl Message<WeeklyReset> for ReminderActor {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        _msg: WeeklyReset,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        major_weekly_reset(self.bot_ref.clone(), ChatId(self.lfg_chat)).await?;
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
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(&mut self, _msg: ScheduleNextMinute, ctx: &mut Context<Self, Self::Reply>) {
        let target_time = (reference_date() + chrono::Duration::minutes(1))
            .with_second(0)
            .unwrap();

        let now = std::time::SystemTime::now();
        let target_system_time =
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(target_time.timestamp() as u64);
        if let Ok(duration) = target_system_time.duration_since(now) {
            tokio::time::sleep(duration).await;
            ctx.actor_ref().tell(Reminders).try_send()?;
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

        let now = std::time::SystemTime::now();
        let target_system_time =
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(target_time.timestamp() as u64);
        if let Ok(duration) = target_system_time.duration_since(now) {
            tokio::time::sleep(duration).await;
            let _ = ctx.actor_ref().tell(DailyReset).await;
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

        let now = std::time::SystemTime::now();
        let target_system_time =
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(target_time.timestamp() as u64);
        if let Ok(duration) = target_system_time.duration_since(now) {
            tokio::time::sleep(duration).await;
            let _ = ctx.actor_ref().tell(WeeklyReset).await;
        }
    }
}
