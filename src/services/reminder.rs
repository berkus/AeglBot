use crate::{datetime::reference_date, Bot, DbConnection};
use diesel::{
    self,
    dsl::{now, IntervalDsl},
    prelude::*,
    sql_types::Timestamptz,
};
use diesel_derives_traits::Model;
use failure::Error;
use futures::Future;
use models::PlannedActivity;
use telebot::{functions::*, RcBot};

pub fn check(bot: &Bot, chat_id: telebot::objects::Integer) -> Result<(), Error> {
    use schema::plannedactivities::dsl::*;

    let reference = reference_date();
    let connection = bot.connection();

    let upcoming_events = plannedactivities
        .filter(start.ge(now.into_sql::<Timestamptz>() - 60_i32.minutes()))
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
        }).collect();

    if upcoming_events.is_empty() {
        return Ok(());
    }

    let text = upcoming_events
        .into_iter()
        .fold("Activities starting soon:\n\n".to_owned(), |acc, event| {
            acc + &format!("{}\n\n", event.display(&connection, None))
        });

    bot.send_html_message(chat_id, text);

    Ok(())
}
