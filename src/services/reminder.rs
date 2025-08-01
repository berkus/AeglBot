use {
    crate::{
        bot_actor::{BotActorMsg, Format, Notify, SendMessage},
        datetime::reference_date,
        models::PlannedActivity,
        BotConnection,
    },
    riker::{actor::Tell, actors::ActorRef},
    teloxide::types::ChatId,
};

pub fn check(bot: ActorRef<BotActorMsg>, connection: BotConnection, chat_id: ChatId) {
    // log::info!("reminder check at {}", reference_date());

    let reference = reference_date();

    let upcoming_events: Vec<PlannedActivity> = PlannedActivity::upcoming_activities(&connection)
        .into_iter()
        .filter(|event| {
            if event.start > reference {
                matches!((event.start - reference).num_minutes(), 60 | 15 | 0)
            } else {
                false
            }
        })
        .collect();

    if upcoming_events.is_empty() {
        return;
    }

    let text = upcoming_events
        .into_iter()
        .fold("Activities starting soon:\n\n".to_owned(), |acc, event| {
            acc + &format!("{}\n\n", event.display(&connection, None))
        });

    bot.tell(SendMessage(text, chat_id, Format::Html, Notify::On), None);
}
