use crate::commands::bot_command::BotCommand;
use diesel::{dsl::*, prelude::*, PgConnection};
use models::PlannedActivity;
use schema::plannedactivities::dsl::*;
use telegram_bot::{self, types::ParseMode, CanReplySendMessage};

pub struct ListCommand;

impl BotCommand for ListCommand {
    fn prefix() -> &'static str {
        "list"
    }

    fn description() -> &'static str {
        "List current events"
    }

    fn execute(
        api: &telegram_bot::Api,
        message: &telegram_bot::Message,
        command: Option<String>,
        name: Option<String>,
        connection: &PgConnection,
    ) {
        let upcoming_events = plannedactivities
            // val hourAgo = DateTime.now(DateTimeZone.forID("Europe/Moscow")).minusHours(1)
            .filter(start.ge(now - 60_i32.minutes()))
            .order(start.asc())
            .load::<PlannedActivity>(connection)
            .expect("TEMP loading @FIXME");

        if upcoming_events.is_empty() {
            api.spawn(message.text_reply("No activities planned, add something with /lfg"));
            return;
        }

        let text = upcoming_events
            .iter()
            .fold("Planned activities:\n\n".to_owned(), |acc, event| {
                acc + &format!("{}\n", event)
            });

        api.spawn(message.text_reply(text).parse_mode(ParseMode::Html));
    }
}
