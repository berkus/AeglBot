// val planned = PlannedActivity
//     .findById(arguments[0].toInt())

// if (planned == null) {
//     sendReply(absSender, chat, "Activity ${arguments[0]} was not found.")
// } else {
//     val member = PlannedActivityMember.find {
//         (PlannedActivityMembers.userId eq dbUser.id) and
//         (PlannedActivityMembers.plannedActivityId eq planned.id)
//     }.singleOrNull()

//     if (member != null) {
//         sendReply(absSender, chat, "You are already part of this fireteam.")
//     } else {
//         if (planned.isFull()) {
//             sendReply(absSender, chat, "This activity fireteam is full.")
//         } else {
//             PlannedActivityMember.new {
//                 this.user = dbUser
//                 this.activity = planned
//             }

//             sendReply(absSender, chat,
//                 dbUser.formatName() + " has joined " + planned.activity.formatName()
//                 +" group " + formatStartTime(planned.start).decapitalize() + "\n"
//                 +planned.membersFormattedList() +" are going\n" + planned.joinPrompt())
//         }
//     }
// }
use crate::commands::{validate_username, BotCommand};
use diesel::PgConnection;
use telegram_bot::{self, CanReplySendMessage};

pub struct JoinCommand;

impl BotCommand for JoinCommand {
    fn prefix() -> &'static str {
        "join"
    }

    fn description() -> &'static str {
        "Join existing fireteam from the list"
    }

    fn execute(
        api: &telegram_bot::Api,
        message: &telegram_bot::Message,
        command: Option<String>,
        team_id: Option<String>,
        connection: &PgConnection,
    ) {
        if team_id.is_none() {
            api.spawn(message.text_reply(
                "To join a fireteam provide fireteam id
Fireteam IDs are available from output of /list command.",
            ));
            return;
        }

        if let Some(user) = validate_username(api, message, connection) {
            // do stuff
            // if activity.too_old() msg(cannot join too old)
        }
        api.spawn(message.text_reply("not implemented yet"));
    }
}
