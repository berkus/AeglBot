use crate::commands::BotCommand;
use diesel::{prelude::*, PgConnection};
use models::ActivityShortcut;
use schema::activityshortcuts::dsl::*;
use telegram_bot::{self, types::ParseMode, CanReplySendMessage};

pub struct ActivitiesCommand;

impl BotCommand for ActivitiesCommand {
    fn prefix() -> &'static str {
        "activities"
    }

    fn description() -> &'static str {
        "List available activity shortcuts"
    }

    fn execute(
        api: &telegram_bot::Api,
        message: &telegram_bot::Message,
        _command: Option<String>,
        _unused: Option<String>,
        connection: &PgConnection,
    ) {
        let games = activityshortcuts
            .select(game)
            .distinct()
            .order(game.asc())
            .load::<String>(connection)
            .expect("TEMP loading @FIXME");

        let mut text = "Activities: use a short name:\n".to_owned();

        for game_name in games.into_iter() {
            text += &format!("*** <b>{0}</b>:\n", game_name);
            let shortcuts = activityshortcuts
                .filter(game.eq(game_name))
                .order(name.asc())
                .load::<ActivityShortcut>(connection)
                .expect("TEMP loading @FIXME");
            for shortcut in shortcuts.into_iter() {
                text += &format!(
                    "<b>{name}</b>\t{link}\n",
                    name = shortcut.name,
                    link = shortcut.link
                );
                // shortcut.link.formatName()
            }
            text += "\n";
        }
        api.spawn(message.text_reply(text).parse_mode(ParseMode::Html));
    }
}
