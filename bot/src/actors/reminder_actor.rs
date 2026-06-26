use {
    crate::actors::bot_actor::{Format, Notify, SendMessage},
    chrono::{DateTime, Timelike, Utc},
    culpa::throws,
    entity::prelude::*,
    kameo::{
        actor::{ActorRef, Spawn},
        prelude::*,
        Actor,
    },
    kameo_actors::scheduler::{Scheduler, SetTimeout},
    libbot::{
        datetime::{d2_reset_time, reference_date, start_at_time, start_at_weekday_time},
        services::destiny_schedule::{this_week_in_d1, this_week_in_d2},
    },
    log::trace,
    sea_orm::DatabaseConnection,
    std::time::Duration,
    teloxide::types::ChatId,
};

// send_at: Instant::now() + delay,
fn schedule_at_time(time: DateTime<Utc>) -> Duration {
    let delay = std::cmp::max(time.timestamp() - Utc::now().timestamp(), 0_i64);
    let delay = Duration::from_secs(delay as u64);
    delay
}

/// Wraps posting scheduled messages to the bot.
#[derive(Actor)]
pub struct ReminderActor {
    bot_ref: ActorRef<crate::actors::bot_actor::BotActor>,
    lfg_chat: i64,
    connection_pool: DatabaseConnection,
    scheduler_ref: ActorRef<Scheduler>,
}

/// Kick off scheduled messages.
pub struct Start;

/// Check for reminders every minute.
pub struct Reminders;

/// Run daily reset at 8pm every day.
pub struct DailyReset;

/// Run weekly reset at 8pm each tuesday.
pub struct WeeklyReset;

impl ReminderActor {
    pub fn new(
        bot_ref: ActorRef<crate::actors::bot_actor::BotActor>,
        lfg_chat: i64,
        connection_pool: DatabaseConnection,
    ) -> Self {
        let scheduler_ref = Scheduler::spawn(Scheduler::new());

        Self {
            bot_ref,
            lfg_chat,
            connection_pool,
            scheduler_ref,
        }
    }

    fn connection(&self) -> &DatabaseConnection {
        &self.connection_pool
    }

    #[throws(anyhow::Error)]
    pub async fn schedule_next_minute(&self, self_ref: WeakActorRef<Self>) {
        let reminders_msg = SetTimeout::new(
            self_ref,
            schedule_at_time(
                (reference_date() + chrono::Duration::minutes(1))
                    .with_second(0)
                    .unwrap(),
            ),
            Reminders,
        );
        self.scheduler_ref.tell(reminders_msg).await?;
    }

    #[throws(anyhow::Error)]
    pub async fn schedule_next_day(&self, self_ref: WeakActorRef<Self>) {
        let daily_reset_msg = SetTimeout::new(
            self_ref,
            schedule_at_time(start_at_time(reference_date(), d2_reset_time())),
            DailyReset,
        );
        self.scheduler_ref.tell(daily_reset_msg).await?;
    }

    #[throws(anyhow::Error)]
    pub async fn schedule_next_week(&self, self_ref: WeakActorRef<Self>) {
        let weekly_reset_msg = SetTimeout::new(
            self_ref,
            schedule_at_time(start_at_weekday_time(
                reference_date(),
                chrono::Weekday::Tue,
                d2_reset_time(),
            )),
            WeeklyReset,
        );
        self.scheduler_ref.tell(weekly_reset_msg).await?;
    }
}

// Schedule first run, the actor handler will reschedule.
impl Message<Start> for ReminderActor {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(&mut self, _msg: Start, ctx: &mut kameo::prelude::Context<Self, Self::Reply>) {
        let self_ref = ctx.actor_ref().downgrade();

        trace!("Scheduling first tick to {self_ref:?}");
        let _ = self.schedule_next_minute(self_ref.clone()).await;
        let _ = self.schedule_next_day(self_ref.clone()).await;
        let _ = self.schedule_next_week(self_ref).await;
    }
}

// Handler for Reminders message
impl Message<Reminders> for ReminderActor {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(
        &mut self,
        _msg: Reminders,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) {
        trace!("Received Reminders");

        let found = PlannedActivities::upcoming_activities_alert(self.connection()).await;

        trace!("Loaded planned activities");

        if let Some(upcoming_events) = found {
            let text = crate::commands::render_events_list(
                &upcoming_events,
                self.connection(),
                None,
                "reminders/upcoming",
            )
            .await?;

            let _ = self
                .bot_ref
                .tell(SendMessage(
                    text,
                    ChatId(self.lfg_chat),
                    Format::Html,
                    Notify::On,
                ))
                .await;
        }

        self.schedule_next_minute(ctx.actor_ref().downgrade())
            .await?;
    }
}

// 1. Daily resets at 20:00 MSK (17:00 UTC) every day
impl Message<DailyReset> for ReminderActor {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(
        &mut self,
        _msg: DailyReset,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) {
        trace!("Received DailyReset");

        self.bot_ref
            .tell(SendMessage(
                "⚡️ Daily reset".into(),
                ChatId(self.lfg_chat),
                Format::Plain,
                Notify::Off,
            ))
            .await?;

        self.schedule_next_day(ctx.actor_ref().downgrade()).await?;
    }
}

// 2. Weekly (main) resets at 20:00 msk every Tue
// 6. On main reset: change in Dreaming City curse
//    dreaming city on 3-week schedule
// 7. On main reset: change in Dreaming City Ascendant Challenges
//    dreaming city challenges on 6-week schedule
impl Message<WeeklyReset> for ReminderActor {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(
        &mut self,
        _msg: WeeklyReset,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) {
        trace!("Received WeeklyReset");

        let msg = format!(
            "⚡️ Weekly reset:\n\n{d1week}\n\n{d2week}",
            d1week = this_week_in_d1(),
            d2week = this_week_in_d2(),
        );
        self.bot_ref
            .tell(SendMessage(
                msg,
                ChatId(self.lfg_chat),
                Format::Markdown,
                Notify::Off,
            ))
            .await?;

        self.schedule_next_week(ctx.actor_ref().downgrade()).await?;
    }
}
