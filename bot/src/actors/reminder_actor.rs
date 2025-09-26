use {
    crate::actors::bot_actor::{Format, Notify, SendMessage},
    chrono::{DateTime, Timelike, Utc},
    culpa::throws,
    entity::prelude::*,
    kameo::{
        actor::{ActorRef, WeakActorRef},
        error::Infallible,
        mailbox, messages,
        prelude::{MailboxReceiver, Message},
        Actor,
    },
    libbot::{
        datetime::{d2_reset_time, reference_date, start_at_time, start_at_weekday_time},
        services::destiny_schedule::{this_week_in_d1, this_week_in_d2},
    },
    log::{error, trace, warn},
    sea_orm::DatabaseConnection,
    std::{
        cmp::Ordering,
        collections::BinaryHeap,
        time::{Duration, Instant},
    },
    teloxide::types::ChatId,
};

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

// #[actor(mailbox = unbounded)]
pub struct ReminderActor {
    bot_ref: ActorRef<crate::actors::bot_actor::BotActor>,
    lfg_chat: i64,
    connection_pool: DatabaseConnection,
    reminders: BinaryHeap<ReminderJob>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum Action {
    Reminders,
    DailyReset,
    WeeklyReset,
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct ReminderJob {
    send_at: Instant,
    action: Action,
}

// For a min-heap (earliest timestamp first)
impl Ord for ReminderJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse the order for min-heap
        other.send_at.cmp(&self.send_at)
    }
}

impl PartialOrd for ReminderJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Actor for ReminderActor {
    type Args = Self;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(args)
    }

    async fn next(
        &mut self,
        actor_ref: WeakActorRef<Self>,
        mailbox_rx: &mut MailboxReceiver<Self>,
    ) -> Option<mailbox::Signal<Self>> {
        loop {
            self.print_reminders();
            // @todo: pop and reinsert, or peek and then pop? which is more robust here?
            if let Some(next) = self.reminders.peek() {
                let duration = next.send_at.duration_since(Instant::now());
                trace!("Sleeping for {duration:?}");
                tokio::select! {
                    biased;
                    signal = mailbox_rx.recv() => {
                        trace!("Woken early from {duration:?} sleep");
                        return signal;
                    },
                    _ = tokio::time::sleep(duration) => {
                        if let Some(next) = self.reminders.pop() {
                            if let Some(actor) = actor_ref.upgrade() {
                                trace!("{duration:?} sleep completed, dispatching action");
                                match next.action {
                                    Action::Reminders => { let _ = actor.tell(Reminders).try_send(); }
                                    Action::DailyReset => { let _ = actor.tell(DailyReset).try_send(); }
                                    Action::WeeklyReset => { let _ = actor.tell(WeeklyReset).try_send(); }
                                }
                            }
                        }
                    }
                }
            } else {
                // No reminder jobs... just receive the next message as usual
                trace!("No reminder jobs, just receive the next message as usual");
                return mailbox_rx.recv().await;
            }
        }
    }

    async fn on_stop(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        reason: kameo::prelude::ActorStopReason,
    ) -> Result<(), Self::Error> {
        warn!("ReminderActor stopped! {reason}");
        Ok(())
    }

    async fn on_panic(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        err: kameo::prelude::PanicError,
    ) -> Result<std::ops::ControlFlow<kameo::prelude::ActorStopReason>, Self::Error> {
        error!("ReminderActor panicked! {err}");
        Ok(std::ops::ControlFlow::Break(
            kameo::prelude::ActorStopReason::Panicked(err),
        ))
    }
}

impl ReminderActor {
    pub fn new(
        bot_ref: ActorRef<crate::actors::bot_actor::BotActor>,
        lfg_chat: i64,
        connection_pool: DatabaseConnection,
    ) -> Self {
        Self {
            bot_ref,
            lfg_chat,
            connection_pool,
            reminders: BinaryHeap::new(),
        }
    }

    fn connection(&self) -> &DatabaseConnection {
        &self.connection_pool
    }

    fn schedule_at_time(&mut self, time: DateTime<Utc>, action: Action) {
        let delay = std::cmp::max(time.timestamp() - Utc::now().timestamp(), 0_i64);
        let delay = Duration::from_secs(delay as u64);

        let job = ReminderJob {
            send_at: Instant::now() + delay,
            action,
        };

        self.reminders.push(job);
    }

    fn print_reminders(&self) {
        let mut heap = self.reminders.clone();
        while let Some(x) = heap.pop() {
            trace!("Reminder: {x:?}");
        }
    }
}

#[messages]
impl ReminderActor {
    #[message]
    pub async fn schedule_next_minute(&mut self) -> anyhow::Result<()> {
        self.schedule_at_time(
            (reference_date() + chrono::Duration::minutes(1))
                .with_second(0)
                .unwrap(),
            Action::Reminders,
        );
        Ok(())
    }

    #[message]
    pub async fn schedule_next_day(&mut self) -> anyhow::Result<()> {
        self.schedule_at_time(
            start_at_time(reference_date(), d2_reset_time()),
            Action::DailyReset,
        );
        Ok(())
    }

    #[message]
    pub async fn schedule_next_week(&mut self) -> anyhow::Result<()> {
        self.schedule_at_time(
            start_at_weekday_time(reference_date(), chrono::Weekday::Tue, d2_reset_time()),
            Action::WeeklyReset,
        );
        Ok(())
    }
}

pub struct Reminders;
pub struct DailyReset;
pub struct WeeklyReset;

impl Message<Reminders> for ReminderActor {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(
        &mut self,
        _msg: Reminders,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) {
        let bot_ref = self.bot_ref.clone();
        let connection = self.connection();
        let lfg_chat = self.lfg_chat;

        trace!("Received Reminders");

        let found = PlannedActivities::upcoming_activities_alert(connection).await;

        trace!("Loaded planned activities");

        if let Some(upcoming_events) = found {
            let text = crate::commands::render_events_list(
                &upcoming_events,
                connection,
                None,
                "reminders/upcoming",
            )
            .await?;

            let _ = bot_ref
                .tell(SendMessage(
                    text,
                    ChatId(lfg_chat),
                    Format::Html,
                    Notify::On,
                ))
                .await;
        }

        ctx.actor_ref().tell(ScheduleNextMinute).try_send()?;
    }
}

impl Message<DailyReset> for ReminderActor {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(
        &mut self,
        _msg: DailyReset,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) {
        daily_reset(self.bot_ref.clone(), ChatId(self.lfg_chat)).await?;
        ctx.actor_ref().tell(ScheduleNextDay).try_send()?;
    }
}

impl Message<WeeklyReset> for ReminderActor {
    type Reply = anyhow::Result<()>;

    #[throws(anyhow::Error)]
    async fn handle(
        &mut self,
        _msg: WeeklyReset,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) {
        major_weekly_reset(self.bot_ref.clone(), ChatId(self.lfg_chat)).await?;
        ctx.actor_ref().tell(ScheduleNextWeek).try_send()?;
    }
}
