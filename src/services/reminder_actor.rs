use {
    crate::{
        datetime::{d2_reset_time, reference_date, start_at_time, start_at_weekday_time},
        services::{destiny_schedule, reminder},
        BotMenu,
    },
    chrono::Timelike,
    riker::{
        actors::{actor, Actor, ActorFactoryArgs, BasicActorRef, Context, Receive, Sender, Tell},
        system::Timer,
    },
    teloxide::types::ChatId,
};

#[actor(
    Reminders,
    DailyReset,
    WeeklyReset,
    ScheduleNextMinute,
    ScheduleNextDay,
    ScheduleNextWeek
)]
pub struct ReminderActor {
    bot: BotMenu,
    lfg_chat: i64,
}

impl Actor for ReminderActor {
    type Msg = ReminderActorMsg;

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}

impl ActorFactoryArgs<(BotMenu, i64)> for ReminderActor {
    fn create_args((bot, lfg_chat): (BotMenu, i64)) -> Self {
        Self { bot, lfg_chat }
    }
}

#[derive(Clone, Debug)]
pub struct Reminders;

#[derive(Clone, Debug)]
pub struct DailyReset;

#[derive(Clone, Debug)]
pub struct WeeklyReset;

impl Receive<Reminders> for ReminderActor {
    type Msg = ReminderActorMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, _msg: Reminders, sender: Sender) {
        reminder::check(&self.bot, ChatId::Id(self.lfg_chat));
        ctx.myself().tell(ScheduleNextMinute, sender);
    }
}

impl Receive<DailyReset> for ReminderActor {
    type Msg = ReminderActorMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, _msg: DailyReset, sender: Sender) {
        destiny_schedule::daily_reset(&self.bot, ChatId::Id(self.lfg_chat));
        ctx.myself().tell(ScheduleNextDay, sender);
    }
}

impl Receive<WeeklyReset> for ReminderActor {
    type Msg = ReminderActorMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, _msg: WeeklyReset, sender: Sender) {
        destiny_schedule::major_weekly_reset(&self.bot, ChatId::Id(self.lfg_chat));
        ctx.myself().tell(ScheduleNextWeek, sender);
    }
}

#[derive(Clone, Debug)]
pub struct ScheduleNextMinute;

#[derive(Clone, Debug)]
pub struct ScheduleNextDay;

#[derive(Clone, Debug)]
pub struct ScheduleNextWeek;

impl Receive<ScheduleNextMinute> for ReminderActor {
    type Msg = ReminderActorMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, _msg: ScheduleNextMinute, _sender: Sender) {
        ctx.schedule_at_time(
            (reference_date() + chrono::Duration::minutes(1))
                .with_second(0)
                .unwrap(),
            ctx.myself(),
            None,
            Reminders,
        );
    }
}

impl Receive<ScheduleNextDay> for ReminderActor {
    type Msg = ReminderActorMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, _msg: ScheduleNextDay, _sender: Sender) {
        ctx.schedule_at_time(
            start_at_time(reference_date(), d2_reset_time()),
            ctx.myself(),
            None,
            DailyReset,
        );
    }
}

impl Receive<ScheduleNextWeek> for ReminderActor {
    type Msg = ReminderActorMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, _msg: ScheduleNextWeek, _sender: Sender) {
        ctx.schedule_at_time(
            start_at_weekday_time(reference_date(), chrono::Weekday::Tue, d2_reset_time()),
            ctx.myself(),
            None,
            WeeklyReset,
        );
    }
}
