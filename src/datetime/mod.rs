use chrono::{prelude::*, Duration, Local};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{Europe::Moscow, Tz};
use std::fmt::Write;

// NaiveDateTime is stored as MSK in the DB

pub fn naive_to_msk(ts: NaiveDateTime) -> DateTime<Tz> {
    Moscow.from_local_datetime(&ts).unwrap()
}

pub fn msk_to_naive(ts: DateTime<Tz>) -> NaiveDateTime {
    ts.with_timezone(&Moscow).naive_local()
}

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

    for item in times.iter() {
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
        format!("just now")
    } else {
        if duration > Duration::zero() {
            format!("in {}", text)
        } else {
            format!("{} ago", text)
        }
    }
}

/// Return today() but in MSK timezone
pub fn reference_date() -> NaiveDateTime {
    Moscow
        .from_utc_datetime(&Utc::now().naive_utc())
        .naive_local()
}

/// Time and reference are in MSK timezone!
///
/// `"Today at 23:00 (starts in 3 hours)"`
pub fn format_start_time(time: NaiveDateTime, reference: NaiveDateTime) -> String {
    let time = naive_to_msk(time);
    let ref_date = naive_to_msk(reference.date().and_hms(0, 0, 0));

    let prefix = if time.date() == ref_date.date() {
        format!("Today")
    } else {
        format!("on {}", time.format("%a %b %e %Y"))
    };

    let prefix2 = time.format("%T");

    let time_diff = time - naive_to_msk(reference);
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
    fn hours_in_msk_match_naive_local() {
        let orig = Moscow.ymd(2018, 9, 4).and_hms(20, 20, 0);
        let db = msk_to_naive(orig);
        assert_eq!(orig.to_string(), "2018-09-04 20:20:00 MSK");
        assert_eq!(db.to_string(), "2018-09-04 20:20:00");
    }

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
        // let hours = 3600;
        // let msk = FixedOffset::east(3 * hours);

        let today = Local::now().naive_local();
        // let today = msk.from_utc_datetime(Utc::now());
        // + Duration::hours(2) + Duration::minutes(30)
        assert_eq!(
            format_start_time(today, reference_date()),
            format!("{}", today.format("Today at %H:%M:%S (started just now)"))
        );
    }
}
