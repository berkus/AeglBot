mod activities_command;
pub use self::activities_command::*;
mod bot_command;
pub use self::bot_command::*;
mod cancel_command;
pub use self::cancel_command::*;
mod details_command;
pub use self::details_command::*;
mod help_command;
pub use self::help_command::*;
mod join_command;
pub use self::join_command::*;
mod lfg_command;
pub use self::lfg_command::*;
mod list_command;
pub use self::list_command::*;
mod psn_command;
pub use self::psn_command::*;
mod whois_command;
pub use self::whois_command::*;

use chrono::{Duration, NaiveDateTime};
use crate::{models::Guardian, schema::guardians::dsl::*};
use diesel::{pg::PgConnection, prelude::*};
use telegram_bot::{self, CanReplySendMessage};

pub fn validate_username(
    api: &telegram_bot::Api,
    message: &telegram_bot::Message,
    connection: &PgConnection,
) -> Option<Guardian> {
    let username = match message.from.username {
        None => {
            api.spawn(message.text_reply(
                "You have no telegram username, register your telegram account first.",
            ));
            return None;
        }
        Some(ref name) => name,
    };
    let db_user = guardians
        .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
        .limit(1)
        .load::<Guardian>(connection);
    match db_user {
        Ok(users) => if users.len() > 0 {
            Some(users[0].clone())
        } else {
            api.spawn(
                message.text_reply("You need to link your PSN account first: use /psn command"),
            );
            None
        },
        Err(_) => {
            api.spawn(message.text_reply("Error querying guardian info."));
            None
        }
    }
}

fn time_diff_string(duration: Duration) -> String {
    let times = vec![
        (Duration::days(365), "year"),
        (Duration::days(30), "month"),
        (Duration::days(1), "day"),
        (Duration::hours(1), "hour"),
        (Duration::minutes(1), "minute"),
    ];

    let mut dur = duration.num_minutes();
    let mut text = "".to_owned();

    for item in times.iter() {
        let (current, timesStr) = item;
        let current = current.num_minutes();
        let temp = (dur / current).abs();

        info!("current {}, dur {}, temp {}", current, dur, temp);

        if temp > 0 {
            dur -= temp * current;
            text += &format!("{} {}{} ", temp, timesStr, if temp != 1 { "s" } else { "" });
        }
    }

    let text = text.trim();

    if text == "" {
        return format!("just now");
    } else {
        if duration > Duration::zero() {
            return format!("in {}", text);
        } else {
            return format!("{} ago", text);
        }
    }
}

// "Today at 23:00 (starts in 3 hours)"
pub fn format_start_time(time: NaiveDateTime) -> String {
    // val prefix = if (time.withTime(0,0,0,0) == DateTime.now().withTime(0,0,0,0)) {
    //     "Today"
    // } else {
    //     "on " + DateTimeFormat.forStyle("S-").print(time)
    // }

    // val prefix2 = DateTimeFormat.forStyle("-S").print(time)

    // val timeDiff = time.getMillis() - DateTime.now().getMillis()
    // val infixStr = if (timeDiff <= 0) { "started" } else { "starts" }

    // return "${prefix} at ${prefix2} (${infixStr} ${timeDiffString(timeDiff)})"
    format!("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_diffs() {
        pretty_env_logger::init();

        assert_eq!(time_diff_string(Duration::minutes(2)), "in 2 minutes");
        assert_eq!(time_diff_string(Duration::minutes(1)), "in 1 minute");
        assert_eq!(time_diff_string(Duration::minutes(0)), "just now");
        assert_eq!(time_diff_string(Duration::minutes(-1)), "1 minute ago");
        assert_eq!(time_diff_string(Duration::minutes(-2)), "2 minutes ago");

        assert_eq!(
            time_diff_string(Duration::days(2) + Duration::hours(15) + Duration::minutes(33)),
            "in 2 days 15 hours 33 minutes"
        );
    }
}
