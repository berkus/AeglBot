use crate::commands::BotCommand;
use diesel::PgConnection;
use telegram_bot::{self, CanReplySendMessage};

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
        command: Option<String>,
        name: Option<String>,
        connection: &PgConnection,
    ) {
        api.spawn(message.text_reply("not implemented yet"));

        // let games = activitiesshortcuts
        //     .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
        //     .limit(1)
        //     .load::<Guardian>(connection);

        // var text = "Activities: use a short name:\n";
        // val games = ActivityShortcuts.slice(ActivityShortcuts.game).selectAll().withDistinct().toList()
        //     .map { game -> game[ActivityShortcuts.game] }.sorted()

        // for (game in games) {
        //     text += "*** <b>${game}</b>:\n" +
        //         ActivityShortcut.find { ActivityShortcuts.game eq game }.toList().sortedBy { ActivityShortcuts.name }.map { act ->
        //             "<b>${act.name}</b>\t${act.link.formatName()}"
        //         }.joinToString("\n") + "\n"
        // }
        // api.spawn(message.text_reply("not implemented yet"));
    }
}
