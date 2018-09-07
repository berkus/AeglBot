use crate::models::{Activity, ActivityShortcut};
use crate::{Bot, BotCommand, DbConnection};
use diesel::{self, prelude::*};
use futures::Future;

pub struct ActivitiesCommand;

impl BotCommand for ActivitiesCommand {
    fn prefix(&self) -> &'static str {
        "/activities"
    }

    fn description(&self) -> &'static str {
        "List available activity shortcuts"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        _unused: Option<String>,
    ) {
        use schema::activities::dsl::{activities, id};
        use schema::activityshortcuts::dsl::{activityshortcuts, game, name};

        let connection = bot.connection();

        let games = activityshortcuts
            .select(game)
            .distinct()
            .order(game.asc())
            .load::<String>(&connection)
            .expect("TEMP loading @FIXME");

        let mut text = "Activities: use a short name:\n".to_owned();

        for game_name in games {
            text += &format!("*** <b>{0}</b>:\n", game_name);
            let shortcuts = activityshortcuts
                .filter(game.eq(game_name))
                .order(name.asc())
                .load::<ActivityShortcut>(&connection)
                .expect("TEMP loading @FIXME");
            for shortcut in shortcuts {
                let link_name = activities
                    .filter(id.eq(shortcut.link))
                    .first::<Activity>(&connection)
                    .expect("TEMP loading @FIXME");

                text += &format!(
                    "<b>{name}</b>\t{link}\n",
                    name = shortcut.name,
                    link = link_name.format_name(),
                );
            }
            text += "\n";
        }

        bot.send_html_reply(&message, text);
    }
}
