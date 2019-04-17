use {
    crate::{
        datetime::{nowtz, reference_date},
        models::PlannedActivity,
        BotMenu, DbConnection,
    },
    anyhow::Result,
    diesel::{
        self,
        dsl::{now, IntervalDsl},
        prelude::*,
    },
    diesel_derives_traits::Model,
    futures::Future,
    teloxide::types::ChatId,
};

pub fn check(bot: &BotMenu, chat_id: ChatId) -> Result<()> {
    use crate::schema::plannedactivities::dsl::*;

    log::info!("reminder check");

    let reference = reference_date();
    let connection = bot.connection();

    let upcoming_events = plannedactivities
        .filter(start.ge(nowtz() - 60_i32.minutes()))
        .order(start.asc())
        .load::<PlannedActivity>(&connection)
        .expect("TEMP loading @FIXME");

    let upcoming_events: Vec<&PlannedActivity> = upcoming_events
        .iter()
        .filter(|event| {
            if event.start > reference {
                match (event.start - reference).num_minutes() {
                    60 | 15 | 0 => true,
                    _ => false,
                }
            } else {
                false
            }
        })
        .collect();

    if upcoming_events.is_empty() {
        return Ok(());
    }

    let text = upcoming_events
        .into_iter()
        .fold("Activities starting soon:\n\n".to_owned(), |acc, event| {
            acc + &format!("{}\n\n", event.display(&connection, None))
        });

    bot.send_html_message_with_notification(chat_id, text);

    Ok(())
}
