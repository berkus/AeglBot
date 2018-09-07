use crate::{commands::validate_username, models::PlannedActivity};
use crate::{Bot, BotCommand, DbConnection};
use diesel::{
    self,
    dsl::{now, IntervalDsl},
    prelude::*,
    sql_types::Timestamptz,
};
use futures::Future;

pub struct ListCommand;

impl BotCommand for ListCommand {
    fn prefix(&self) -> &'static str {
        "/list"
    }

    fn description(&self) -> &'static str {
        "List current events"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _args: Option<String>,
    ) {
        use schema::plannedactivities::dsl::*;

        let connection = bot.connection();

        let upcoming_events = plannedactivities
            .filter(start.ge(now.into_sql::<Timestamptz>() - 60_i32.minutes()))
            .order(start.asc())
            .load::<PlannedActivity>(&connection)
            .expect("TEMP loading @FIXME");

        if upcoming_events.is_empty() {
            return bot.send_plain_reply(
                &message,
                "No activities planned, add something with /lfg".into(),
            );
        }

        if let Some(guardian) = validate_username(bot, &message, &connection) {
            let text = upcoming_events
                .iter()
                .fold("Planned activities:\n\n".to_owned(), |acc, event| {
                    acc + &format!("{}\n\n", event.display(&connection, Some(&guardian)))
                });

            bot.send_html_reply(&message, text);
        }
    }
}
