use {
    chrono::{prelude::*, DateTime, Duration, TimeZone, Utc},
    chrono_tz::{Europe::Moscow, Tz},
    diesel::{helper_types::AsExprOf, sql_types::Timestamptz},
    std::fmt::Write,
};

// Diesel farts, see issues/1752
pub fn nowtz() -> AsExprOf<diesel::dsl::now, Timestamptz> {
    use diesel::{dsl::now, IntoSql};
    now.into_sql::<Timestamptz>()
}

// All internal date representation and storage is in UTC.
// MSK time used only to parse input time and to display times.

pub type BotDateTime = chrono::DateTime<chrono::Utc>;
pub type BotTime = chrono::NaiveTime; // UTC

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
            )
            .unwrap();
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

pub fn format_uptime() -> String {
    time_diff_string(reference_date() - bot_start_time())
}

pub fn bot_start_time() -> BotDateTime {
    use std::sync::LazyLock;
    static START_TIME: LazyLock<BotDateTime> = LazyLock::new(reference_date);
    *START_TIME
}

/// Return today() but in UTC timezone
pub fn reference_date() -> BotDateTime {
    Utc::now()
}

/// Destiny reset time in UTC.
/// Resets are at 17:00 UTC.
pub fn d2_reset_time() -> BotTime {
    BotTime::from_hms_opt(17, 0, 0).unwrap()
}

/// Display time in Moscow timezone (MSK)
pub fn display_time(t: BotDateTime) -> DateTime<Tz> {
    Moscow.from_utc_datetime(&t.naive_utc())
}

/// Replace the time in the given datatime.
fn time_override(now: BotDateTime, start: BotTime) -> BotDateTime {
    now.with_hour(start.hour())
        .unwrap()
        .with_minute(start.minute())
        .unwrap()
        .with_second(start.second())
        .unwrap()
}

// Instant at which the tokio_timer stream should start producing events.
// With help of @Douman on https://gitter.im/tokio-rs/tokio:
// https://gitlab.com/Douman/snow-white/blob/master/src/system/discord.rs#L377-397
// This fn calculates only offset to be testable, the public fn `start_at_time` uses it.
fn start_at_time_offset(now: BotDateTime, start: BotTime) -> Duration {
    let now_time = now.time();

    let first = if now_time > start {
        time_override(now + Duration::days(1), start)
    } else {
        time_override(now, start)
    };

    first - now
}

pub fn start_at_time(now: BotDateTime, start: BotTime) -> BotDateTime {
    reference_date() + start_at_time_offset(now, start)
}

// For weekly events - start on given day of week, at a given time.
// This fn calculates only offset to be testable, the public fn `start_at_weekday_time` uses it.
fn start_at_weekday_time_offset(now: BotDateTime, wd: chrono::Weekday, start: BotTime) -> Duration {
    let first = if wd.number_from_monday() < now.weekday().number_from_monday() {
        // That weekday passed, schedule for next week
        let num_days = (7 - now.weekday().number_from_monday() + wd.number_from_monday()) as i64;
        time_override(now, start) + Duration::days(num_days)
    } else {
        let num_days = (wd.number_from_monday() - now.weekday().number_from_monday()) as i64;
        // The day is right, but time has passed - schedule to next week
        if num_days == 0 && now.time() > start {
            time_override(now, start) + Duration::weeks(1)
        } else {
            time_override(now, start) + Duration::days(num_days)
        }
    };

    first - now
}

pub fn start_at_weekday_time(now: BotDateTime, wd: chrono::Weekday, start: BotTime) -> BotDateTime {
    reference_date() + start_at_weekday_time_offset(now, wd, start)
}

/// Time and reference are in UTC? timezone!
///
/// `"Today at 23:00 (starts in 3 hours)"`
pub fn format_start_time(time: BotDateTime, reference: BotDateTime) -> String {
    let ref_date = reference.date_naive().and_hms_opt(0, 0, 0).unwrap(); // @todo don't unwrap here, date may be invalid

    let prefix = if time.date_naive() == ref_date.date() {
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

    #[test]
    fn test_start_at_time() {
        let ref_time = Utc.with_ymd_and_hms(2018, 10, 24, 17, 30, 0).unwrap();

        assert_eq!(
            start_at_time_offset(ref_time, BotTime::from_hms_opt(17, 0, 0).unwrap()),
            Duration::hours(23) + Duration::minutes(30)
        );
        assert_eq!(
            start_at_time_offset(ref_time, BotTime::from_hms_opt(17, 30, 0).unwrap()),
            Duration::seconds(0)
        );
        assert_eq!(
            start_at_time_offset(ref_time, BotTime::from_hms_opt(18, 30, 0).unwrap()),
            Duration::hours(1)
        );
    }

    #[test]
    fn test_start_at_weekday() {
        // It's wednesday
        let ref_date = Utc.with_ymd_and_hms(2018, 10, 24, 17, 30, 0).unwrap(); // Wednesday

        // We schedule on wednesday later - wait just that
        assert_eq!(
            start_at_weekday_time_offset(
                ref_date,
                Weekday::Wed,
                BotTime::from_hms_opt(18, 0, 0).unwrap()
            ),
            Duration::minutes(30)
        );
        // We schedule on wednesday but before - wait almost 1 week
        assert_eq!(
            start_at_weekday_time_offset(
                ref_date,
                Weekday::Wed,
                BotTime::from_hms_opt(17, 0, 0).unwrap()
            ),
            Duration::days(6) + Duration::hours(23) + Duration::minutes(30)
        );
        // We schedule on friday, same time - wait till friday (2 days)
        assert_eq!(
            start_at_weekday_time_offset(
                ref_date,
                Weekday::Fri,
                BotTime::from_hms_opt(17, 30, 0).unwrap()
            ),
            Duration::days(2)
        );
        // We schedule on friday, earlier time - wait till friday (almost 2 days)
        assert_eq!(
            start_at_weekday_time_offset(
                ref_date,
                Weekday::Fri,
                BotTime::from_hms_opt(17, 00, 0).unwrap()
            ),
            Duration::days(1) + Duration::hours(23) + Duration::minutes(30)
        );
        // We schedule on friday, later time - wait till friday (over 2 days)
        assert_eq!(
            start_at_weekday_time_offset(
                ref_date,
                Weekday::Fri,
                BotTime::from_hms_opt(18, 30, 0).unwrap()
            ),
            Duration::days(2) + Duration::hours(1)
        );
        // We schedule on monday, same time - wait till monday (5 days)
        assert_eq!(
            start_at_weekday_time_offset(
                ref_date,
                Weekday::Mon,
                BotTime::from_hms_opt(17, 30, 0).unwrap()
            ),
            Duration::days(5)
        );
        // We schedule on monday, earlier time - wait till monday (almost 5 days)
        assert_eq!(
            start_at_weekday_time_offset(
                ref_date,
                Weekday::Mon,
                BotTime::from_hms_opt(17, 00, 0).unwrap()
            ),
            Duration::days(4) + Duration::hours(23) + Duration::minutes(30)
        );
        // We schedule on monday, later time - wait till monday (over 5 days)
        assert_eq!(
            start_at_weekday_time_offset(
                ref_date,
                Weekday::Mon,
                BotTime::from_hms_opt(18, 30, 0).unwrap()
            ),
            Duration::days(5) + Duration::hours(1)
        );
    }
}
