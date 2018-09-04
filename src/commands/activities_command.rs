use crate::{
    commands::{bot_command::BotCommand, send_html_reply},
    models::{Activity, ActivityShortcut},
};
use diesel::{self, pg::PgConnection, prelude::*};
use futures::Future;
use telebot::{functions::*, RcBot};

pub struct ActivitiesCommand;

impl BotCommand for ActivitiesCommand {
    fn prefix() -> &'static str {
        "activities"
    }

    fn description() -> &'static str {
        "List available activity shortcuts"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        _unused: Option<String>,
        connection: &PgConnection,
    ) {
        use schema::activities::dsl::{activities, id};
        use schema::activityshortcuts::dsl::{activityshortcuts, game, name};

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
                let link_name = activities
                    .filter(id.eq(shortcut.link))
                    .first::<Activity>(connection)
                    .expect("TEMP loading @FIXME");

                text += &format!(
                    "<b>{name}</b>\t{link}\n",
                    name = shortcut.name,
                    link = link_name.format_name(),
                );
            }
            text += "\n";
        }

        send_html_reply(bot, message, text);
    }
}
