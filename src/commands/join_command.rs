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
use crate::commands::{send_plain_reply, validate_username, BotCommand};
use diesel::{self, associations::HasTable, pg::PgConnection, prelude::*};
use diesel_derives_traits::{Model, NewModel};
use futures::Future;
use models::{Activity, ActivityShortcut, NewPlannedActivity, NewPlannedActivityMember};
use telebot::{functions::*, RcBot};

pub struct JoinCommand;

impl JoinCommand {
    fn usage(bot: &RcBot, message: telebot::objects::Message) {
        send_plain_reply(
            bot,
            &message,
            "To join a fireteam provide fireteam id
Fireteam IDs are available from output of /list command."
                .into(),
        );
    }
}

impl BotCommand for JoinCommand {
    fn prefix() -> &'static str {
        "join"
    }

    fn description() -> &'static str {
        "Join existing activity from the list"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        team_id: Option<String>,
        connection: &PgConnection,
    ) {
        if team_id.is_none() {
            return JoinCommand::usage(bot, message);
        }

        if let Some(_user) = validate_username(bot, &message, connection) {
            // do stuff
            // if activity.too_old() msg(cannot join too old)
        }
        send_plain_reply(bot, &message, "not implemented yet".into());
    }
}
