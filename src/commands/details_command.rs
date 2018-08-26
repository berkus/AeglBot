//     override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
//     {
//         if (arguments.size < 2) {
//             sendReply(absSender, chat, "To update fireteam details enter /details ID freeform text\n"
//             + "To delete details use /details ID del.\n"
//             + "Fireteam IDs are available from output of /list command.")
//             return
//         }

//         transaction {
//             logger.addLogger(Slf4jSqlLogger())

//             val dbUser = Guardian.find { Guardians.telegramName eq user.getUserName() }.singleOrNull()

//             if (dbUser == null) {
//                 sendReply(absSender, chat, "You need to link your PSN account first: use /psn command")
//             } else {

//                 val planned = PlannedActivity
//                     .findById(arguments[0].toInt())

//                 if (planned == null) {
//                     sendReply(absSender, chat, "Activity ${arguments[0]} was not found.")
//                 } else {

//                     planned.details = if (arguments[1] == "del") { "" }
//                                       else { arguments.drop(1).joinToString(" ") }

//                     sendReply(absSender, chat, "Details updated.")
//                 }
//             }
//         }
//     }
use crate::commands::BotCommand;
use diesel::PgConnection;
use telegram_bot::{self, CanReplySendMessage};

pub struct DetailsCommand;

impl BotCommand for DetailsCommand {
    fn prefix() -> &'static str {
        "details"
    }

    fn description() -> &'static str {
        "Set group details as freeform text"
    }

    fn execute(
        api: &telegram_bot::Api,
        message: &telegram_bot::Message,
        command: Option<String>,
        name: Option<String>,
        connection: &PgConnection,
    ) {
        api.spawn(message.text_reply("not implemented yet"));
    }
}
