use {
    crate::{
        commands::{decapitalize, validate_guardian},
        datetime::{format_start_time, reference_date},
        models::PlannedActivity,
    },
    chrono::Duration,
    diesel_derives_traits::Model,
    teloxide::prelude::*,
};

// command_actor!(CancelCommand, [ActorUpdateMessage]);

pub type ActivityId = i32;

fn send_reply<S>(bot: Bot, message: &CommandMsg, reply: S) -> Result<(), ActorProcessingErr>
where
    S: Into<String>,
{
    bot.SendMessageReply(reply.into(), message.clone(), Format::Plain, Notify::Off);
    Ok(())
}

fn usage(message: &CommandMsg) -> HandlerResult {
    send_reply(
        message,
        "To leave a fireteam provide fireteam id
 Fireteam IDs are available from output of /list command.",
    )
}

pub async fn cancel_command(
    bot: Bot,
    connection: DbConnection,
    message: Message,
    activity_id: ActivityId,
) -> HandlerResult {
    if let Some(guardian) = validate_guardian(&self.bot_ref, &message, &connection) {
        let planned =
            PlannedActivity::find_one(&connection, &activity_id).expect("Failed to run SQL");

        if planned.is_none() {
            return send_reply(&message, format!("Activity {} was not found.", activity_id));
        }

        let planned = planned.unwrap();

        let member = planned.find_member(&connection, &guardian);

        if member.is_none() {
            return send_reply(&message, "You are not part of this group.");
        }

        if planned.start < reference_date() - Duration::hours(1) {
            return send_reply(&message, "You can not leave past activities.");
        }

        let member = member.unwrap();

        if member.destroy(&connection).is_err() {
            return send_reply(&message, "Failed to remove group member");
        }

        let act_name = planned.activity(&connection).format_name();
        let act_time = decapitalize(&format_start_time(planned.start, reference_date()));

        let suffix = if planned.members(&connection).is_empty() {
            if planned.destroy(&connection).is_err() {
                return send_reply(&message, "Failed to remove planned activity");
            }
            "This fireteam is disbanded and can no longer be joined.".into()
        } else {
            format!(
                "{} are going
 {}",
                planned.members_formatted_list(&connection),
                planned.join_prompt(&connection)
            )
        };

        send_reply(
            &message,
            format!(
                "{guarName} has left {actName} group {actTime}
 {suffix}",
                guarName = guardian.format_name(),
                actName = act_name,
                actTime = act_time,
                suffix = suffix
            ),
        )?;
    }
    Ok(())
}
