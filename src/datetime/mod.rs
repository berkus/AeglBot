use chrono::{prelude::*, Duration, Local};
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{Europe::Moscow, Tz};
use std::fmt::Write;

// All internal date representation and storage is in UTC.
// MSK time used only to parse input time and to display times.

pub type BotDateTime = chrono::DateTime<chrono::Utc>;

fn time_diff_string(duration: Duration) -> String {
    let times = vec![
        (Duration::days(365), "year"),
        (Duration::days(30), "month"),
        (Duration::days(1), "day"),
        (Duration::hours(1), "hour"),
        (Duration::minutes(1), "minute"),
    ];

    let mut dur = duration.num_minutes().abs();
    let mut text = String::new();

    for item in &times {
        let (current, times_str) = item;
        let current = current.num_minutes();
        let temp = dur / current;

        if temp > 0 {
            dur -= temp * current;
            write!(
                &mut text,
                "{} {}{} ",
                temp,
                times_str,
                if temp != 1 { "s" } else { "" }
            );
        }
    }

    let text = text.trim();

    if text.is_empty() {
        "just now".to_string()
    } else if duration > Duration::zero() {
        format!("in {}", text)
    } else {
        format!("{} ago", text)
    }
}

/// Return today() but in MSK timezone
pub fn reference_date() -> BotDateTime {
    Utc::now()
}

pub fn display_time(t: BotDateTime) -> DateTime<Tz> {
    Moscow.from_utc_datetime(&t.naive_utc())
}

/// Time and reference are in MSK timezone!
///
/// `"Today at 23:00 (starts in 3 hours)"`
pub fn format_start_time(time: BotDateTime, reference: BotDateTime) -> String {
    let ref_date = reference.date().and_hms(0, 0, 0);

    let prefix = if time.date() == ref_date.date() {
        "Today".to_string()
    } else {
        format!("on {}", time.format("%a %b %e %Y"))
    };

    let prefix2 = display_time(time).format("%T");

    let time_diff = time - reference;
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
            time_diff_string(Duration::minutes(-67)),
            "1 hour 7 minutes ago"
        );

        assert_eq!(
            time_diff_string(Duration::days(2) + Duration::hours(15) + Duration::minutes(33)),
            "in 2 days 15 hours 33 minutes"
        );
    }

    #[test]
    fn test_start_time_formats() {
        let today = reference_date();
        // let today = msk.from_utc_datetime(Utc::now());
        let msk_time = Moscow.from_utc_datetime(&today.naive_utc());
        // + Duration::hours(2) + Duration::minutes(30)
        assert_eq!(
            format_start_time(today, reference_date()),
            format!(
                "{}",
                msk_time.format("Today at %H:%M:%S (started just now)")
            )
        );
    }
}
