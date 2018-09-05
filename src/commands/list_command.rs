use crate::DbConnection;
use crate::{
    commands::{bot_command::BotCommand, send_html_reply, send_plain_reply, validate_username},
    models::PlannedActivity,
};
use diesel::{
    self,
    dsl::{now, IntervalDsl},
    prelude::*,
    sql_types::Timestamptz,
};
use futures::Future;
use telebot::{functions::*, RcBot};

pub struct ListCommand;

impl BotCommand for ListCommand {
    fn prefix() -> &'static str {
        "list"
    }

    fn description() -> &'static str {
        "List current events"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        _args: Option<String>,
        connection: &DbConnection,
    ) {
        use schema::plannedactivities::dsl::*;

        let upcoming_events = plannedactivities
            .filter(start.ge(now.into_sql::<Timestamptz>() - 60_i32.minutes()))
            .order(start.asc())
            .load::<PlannedActivity>(connection)
            .expect("TEMP loading @FIXME");

        if upcoming_events.is_empty() {
            send_plain_reply(
                bot,
                &message,
                "No activities planned, add something with /lfg".into(),
            );
            return;
        }

        if let Some(guardian) = validate_username(bot, &message, connection) {
            let text = upcoming_events
                .iter()
                .fold("Planned activities:\n\n".to_owned(), |acc, event| {
                    acc + &format!("{}\n\n", event.display(connection, Some(&guardian)))
                });

            send_html_reply(bot, &message, text);
        }
    }
}
