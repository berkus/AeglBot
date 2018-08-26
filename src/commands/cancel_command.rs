//     override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
//     {
//         if (arguments.size != 1) {
//             sendReply(absSender, chat, "To leave a fireteam provide fireteam id\n"
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
//                     val member = PlannedActivityMember.find {
//                         (PlannedActivityMembers.userId eq dbUser.id) and
//                         (PlannedActivityMembers.plannedActivityId eq planned.id)
//                     }.singleOrNull()

//                     if (member == null) {
//                         sendReply(absSender, chat, "You are not part of this group.")
//                     } else {
//                         member.delete()

//                         var suffix = planned.membersFormattedList() +" are going\n"+
//                                      planned.joinPrompt()

//                         if (planned.members.count() == 0) {
//                             planned.delete()
//                             suffix = "This fireteam is disbanded and can no longer be joined."
//                         }

//                         sendReply(absSender, chat,
//                             dbUser.formatName() + " has left " + planned.activity.formatName()
//                             + " group " + formatStartTime(planned.start).decapitalize() + "\n"
//                             +suffix)
//                     }
//                 }
//             }
//         }
//     }
use crate::commands::BotCommand;
use diesel::PgConnection;
use telegram_bot::{self, CanReplySendMessage};

pub struct CancelCommand;

impl BotCommand for CancelCommand {
    fn prefix() -> &'static str {
        "cancel"
    }

    fn description() -> &'static str {
        "Cancel joining fireteam"
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
