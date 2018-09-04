use crate::commands::{send_plain_reply, validate_username, BotCommand};
use diesel::{self, associations::HasTable, pg::PgConnection, prelude::*};
use diesel_derives_traits::{Model, NewModel};
use futures::Future;
use models::{Activity, PlannedActivity, PlannedActivityMember};
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
        connection: &PgConnection,
    ) {
        if activity_id.is_none() {
            return CancelCommand::usage(bot, message);
        }

        let activity_id = activity_id.unwrap().parse::<i32>();
        if let Err(_) = activity_id {
            return CancelCommand::usage(bot, message);
        }

        let activity_id = activity_id.unwrap();

        if let Some(guardian) = validate_username(bot, &message, connection) {
            let planned =
                PlannedActivity::find_one(connection, &activity_id).expect("Failed to run SQL");

            if planned.is_none() {
                return send_plain_reply(
                    bot,
                    &message,
                    format!("Activity {} was not found.", activity_id),
                );
            }

            let planned = planned.unwrap();

            let member = planned.find_member(connection, guardian);

            if member.is_none() {
                return send_plain_reply(bot, &message, "You are not part of this group.".into());
            }

            let member = member.unwrap();

            // if activity.too_old(2.hours) msg(cannot cancel too old)

            member.destroy(connection);

            // var suffix = planned.membersFormattedList() +" are going\n"+
            //              planned.joinPrompt()

            // if (planned.members.count() == 0) {
            //     planned.delete()
            //     suffix = "This fireteam is disbanded and can no longer be joined."
            // }

            // sendReply(absSender, chat,
            //     dbUser.formatName() + " has left " + planned.activity.formatName()
            //     + " group " + formatStartTime(planned.start).decapitalize() + "\n"
            //     +suffix)

            // if activity.too_old() msg(cannot join too old)
        }
    }
}
