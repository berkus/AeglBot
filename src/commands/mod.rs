// mod activities_command;
// pub use self::activities_command::*;
mod bot_command;

pub use self::bot_command::*;

// mod cancel_command;
// pub use self::cancel_command::*;
// mod details_command;
// pub use self::details_command::*;
// mod help_command;
// pub use self::help_command::*;
// mod join_command;
// pub use self::join_command::*;
// mod lfg_command;
// pub use self::lfg_command::*;
// mod list_command;
// pub use self::list_command::*;
mod psn_command;
pub use self::psn_command::*;
mod whois_command;
pub use self::whois_command::*;

use chrono::{prelude::*, Duration, Local};
use crate::{models::Guardian, schema::guardians::dsl::*};
use diesel::{pg::PgConnection, prelude::*};
use futures::Future;
use telebot::{functions::*, RcBot};

pub fn spawn_message(bot: &RcBot, m: WrapperMessage) {
    bot.inner
        .handle
        .spawn(m.send().map(|_| ()).map_err(|e| error!("Error: {:?}", e)));
}

pub fn validate_username(
    bot: &RcBot,
    message: &telebot::objects::Message,
    connection: &PgConnection,
) -> Option<Guardian> {
    let username = match message.from.clone().unwrap().username {
        None => {
            spawn_message(
                bot,
                bot.message(
                    message.chat.id,
                    "You have no telegram username, register your telegram account first.".into(),
                ).reply_to_message_id(message.message_id),
            );
            return None;
        }
        Some(ref name) => name.clone(),
    };

    let db_user = guardians
        .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
        .limit(1)
        .load::<Guardian>(connection);
    match db_user {
        Ok(users) => if users.len() > 0 {
            Some(users[0].clone())
        } else {
            spawn_message(
                bot,
                bot.message(
                    message.chat.id,
                    "You need to link your PSN account first: use /psn command".into(),
                ).reply_to_message_id(message.message_id),
            );
            None
        },
        Err(_) => {
            spawn_message(
                bot,
                bot.message(message.chat.id, "Error querying guardian info.".into())
                    .reply_to_message_id(message.message_id),
            );
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
        let (current, times_str) = item;
        let current = current.num_minutes();
        let temp = (dur / current).abs();

        if temp > 0 {
            dur -= temp * current;
            text += &format!(
                "{} {}{} ",
                temp,
                times_str,
                if temp != 1 { "s" } else { "" }
            );
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
pub fn format_start_time(time: DateTime<Local>) -> String {
    let prefix = if time.date() == Local::today() {
        //@fixme Date<MskTimeZone>
        format!("Today")
    } else {
        format!("on {}", time.format("%a %b %e %Y"))
    };

    let prefix2 = time.format("%T");

    let time_diff = time - Local::now();
    let infix_str = if time_diff < Duration::zero() {
        "started"
    } else {
        "starts"
    };

    format!(
        "{} at {} ({} {})",
        prefix,
        prefix2,
        infix_str,
        time_diff_string(time_diff)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_diffs() {
        assert_eq!(time_diff_string(Duration::minutes(2)), "in 2 minutes");
        assert_eq!(time_diff_string(Duration::minutes(1)), "in 1 minute");
        assert_eq!(time_diff_string(Duration::minutes(0)), "just now");
        assert_eq!(time_diff_string(Duration::seconds(20)), "just now");
        assert_eq!(time_diff_string(Duration::minutes(-1)), "1 minute ago");
        assert_eq!(time_diff_string(Duration::minutes(-2)), "2 minutes ago");

        assert_eq!(
            time_diff_string(Duration::days(2) + Duration::hours(15) + Duration::minutes(33)),
            "in 2 days 15 hours 33 minutes"
        );
    }

    #[test]
    fn test_start_time_formats() {
        // let hours = 3600;
        // let msk = FixedOffset::east(3 * hours);

        let today = Local::now();
        // let today = msk.from_utc_datetime(Utc::now());
        // + Duration::hours(2) + Duration::minutes(30)
        assert_eq!(
            format_start_time(today),
            format!("{}", today.format("Today at %H:%M:%S (started just now)"))
        );
    }
}
