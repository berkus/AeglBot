// val minutesAgo = DateTime.now(DateTimeZone.forID("Europe/Moscow")).minusMinutes(15)
// val objs = PlannedActivity.find {
//         PlannedActivities.start greaterEq minutesAgo
//     }.toList().sortedBy { it.start }.map { act ->
//         "<b>"+act.id+"</b>: <b>"+act.activity.formatName()+"</b>\n" +
//             act.detailsFormatted() +
//             act.membersFormattedColumn() + "\n" +
//             //"<b>" + formatStartTime(act.start) + "</b>\n" +
//             act.joinPrompt() + "\n"
//     }.joinToString("\n")

// if ("".equals(objs)) {
// } else {
//     print(objs)
//     //sendReply(absSender, chat, "Planned activities:\n\n" + objs, true)
// }
use chrono::NaiveDateTime;
use crate::commands::send_html_message;
use crate::DbConnection;
use diesel::prelude::*;
use diesel_derives_traits::Model;
use failure::Error;
use futures::Future;
use telebot::{functions::*, RcBot};

pub fn check(
    bot: &RcBot,
    chat_id: telebot::objects::Integer,
    connection: &DbConnection,
) -> Result<(), Error> {
    use schema::plannedactivities::dsl::*;

    let reference = reference_date();

    let upcoming_events = plannedactivities
        .filter(start.ge(now - 60_i32.minutes())) // FIXME this will sort based on DB local TZ
        .order(start.asc())
        .load::<PlannedActivity>(connection)
        .expect("TEMP loading @FIXME");

    let upcoming_events: Vec<&PlannedActivity> = upcoming_events
        .iter()
        .filter(|event| match (event.start - reference).num_minutes() {
            60 | 30 | 15 | 5 => true,
            _ => false,
        }).collect();

    if upcoming_events.is_empty() {
        return Ok(());
    }

    let text = upcoming_events
        .into_iter()
        .fold("Activities starting soon:\n\n".to_owned(), |acc, event| {
            acc + &format!("{}\n\n", event.display(connection, None))
        });

    send_html_message(bot, chat_id, text);

    Ok(())
}
