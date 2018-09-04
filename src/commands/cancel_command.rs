//         transaction {
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
use crate::commands::{send_plain_reply, BotCommand};
use diesel::{self, associations::HasTable, pg::PgConnection, prelude::*};
use diesel_derives_traits::{Model, NewModel};
use futures::Future;
use models::{Activity, ActivityShortcut, NewPlannedActivity, NewPlannedActivityMember};
use telebot::{functions::*, RcBot};

pub struct CancelCommand;

impl CancelCommand {
    fn usage(bot: &RcBot, message: telebot::objects::Message) {
        send_plain_reply(
            bot,
            &message,
            "To leave a fireteam provide fireteam id
Fireteam IDs are available from output of /list command."
                .into(),
        );
    }
}

impl BotCommand for CancelCommand {
    fn prefix() -> &'static str {
        "cancel"
    }

    fn description() -> &'static str {
        "Leave joined activity"
    }

    fn execute(
        bot: &RcBot,
        message: telebot::objects::Message,
        _command: Option<String>,
        activity_id: Option<String>,
        _connection: &PgConnection,
    ) {
        if activity_id.is_none() {
            return CancelCommand::usage(bot, message);
        }

        let id = activity_id.unwrap().parse::<u32>();
        if let Err(_) = id {
            return CancelCommand::usage(bot, message);
        }

        send_plain_reply(bot, &message, "not implemented yet".into());
    }
}
