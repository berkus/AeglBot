use {
    crate::{
        bot_actor::{Format, Notify, SendMessage},
        datetime::reference_date,
        BotConnection,
    },
    entity::plannedactivities,
    kameo::prelude::*,
    sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
    teloxide::types::ChatId,
};

pub async fn check(
    bot: ActorRef<crate::bot_actor::BotActor>,
    connection: BotConnection,
    chat_id: ChatId,
) {
    // log::info!("reminder check at {}", reference_date());

    let reference = reference_date();

    let upcoming_events = plannedactivities::Entity::find()
        .filter(plannedactivities::Column::Start.gt(reference))
        .all(&connection)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|event| {
            let event_start: chrono::DateTime<chrono::Utc> = event.start.into();
            if event_start > reference {
                let diff = event_start - reference;
                matches!(diff.num_minutes(), 60 | 15 | 0)
            } else {
                false
            }
        })
        .collect::<Vec<_>>();

    if upcoming_events.is_empty() {
        return;
    }

    let text = upcoming_events
        .into_iter()
        .fold("Activities starting soon:\n\n".to_owned(), |acc, event| {
            acc + &format!("Activity {} starting soon\n\n", event.id)
        });

    let _ = bot
        .tell(SendMessage(text, chat_id, Format::Html, Notify::On))
        .await;
}
