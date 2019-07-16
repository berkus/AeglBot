use crate::{
    datetime::{reference_date, BotDateTime},
    Bot, DbConnection,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use failure::Error;
use futures::Future;
use telebot::{functions::*, RcBot};

// Destiny 2 schedules on tracking:

// 1. Daily resets at 20:00 msk each day
pub fn daily_reset(bot: &Bot, chat_id: telebot::objects::Integer) -> Result<(), Error> {
    bot.send_plain_message(chat_id, "⚡️ Daily reset".into());
    Ok(())
}

pub fn dreaming_city_cycle() -> String {
    let curses: [&'static str; 3] = ["Weak Curse", "Growing Curse", "Strongest Curse"];
    let urls: [&'static str; 3] = [
        "https://www.youtube.com/watch?v=6tJZXAa57fY",
        "https://www.youtube.com/watch?v=7WvxeOnhClY",
        "https://www.youtube.com/watch?v=Bwgwa6HpXTI",
    ];
    let dc_week = dc_week_number(reference_date()) as usize;
    format!(
        "💫 Dreaming City: {} ([Ascendant Chests]({}))\n{}",
        curses[dc_week],
        urls[dc_week],
        if dc_week == 2 {
            "(Shattered Throne is available)".to_string()
        } else {
            format!(
                "(Shattered Throne will become available in {} {})",
                2 - dc_week,
                if 2 - dc_week == 1 { "week" } else { "weeks" }
            )
        }
    )
}

pub fn escalation_protocol_cycle() -> String {
    let bosses: [&'static str; 5] = [
        "💀 Nur Abath, Crest of Xol\n⚔️ Shotgun",
        "💀 Kathok, Roar of Xol\n⚔️ SMG",
        "💀 Domkath, the Mask\n⚔️ Sniper Rifle",
        "💀 Naksud, the Famine\n⚔️ Shotgun, SMG, Sniper Rifle",
        "💀 Bok Litur, the Hunger of Xol\n⚔️ Shotgun, SMG, Sniper Rifle",
    ];

    let proto_week = protocol_week_number(reference_date()) as usize;
    format!("Escalation Protocol:\n{}", bosses[proto_week])
}

// 2. Weekly (main) resets at 20:00 msk every Tue
// 5. On main reset: change in Protocol boss drops
//    protocol on 5-week schedule
// 6. On main reset: change in Dreaming City curse
//    dreaming city on 3-week schedule
//   6a. on Strongest Curse week the Shattered Throne is available
pub fn major_weekly_reset(bot: &Bot, chat_id: telebot::objects::Integer) -> Result<(), Error> {
    let msg = format!(
        "Weekly Reset:\n\n{}\n\n{}",
        dreaming_city_cycle(),
        escalation_protocol_cycle()
    );
    bot.send_md_message(chat_id, msg);
    Ok(())
}

// 3. Weekly (minor) resets at 20:00 msg every Fri
//   3a. Whisper of the Worm becomes available
pub fn minor_weekly_reset(_bot: &Bot, _chat_id: telebot::objects::Integer) -> Result<(), Error> {
    // bot.send_plain_message(chat_id, "Whisper of the Worm mission now available".into());
    Ok(())
}

// 4. Monday 20:00 msg end of Whisper of the Worm quest
pub fn end_of_weekend(_bot: &Bot, _chat_id: telebot::objects::Integer) -> Result<(), Error> {
    // bot.send_html_message(
    //     chat_id,
    //     "Whisper of the Worm mission is not available until next weekend".into(),
    // );
    Ok(())
}

// a. need to calculate current d2 week number
fn dc_week_number(now: BotDateTime) -> i64 {
    lazy_static! {
        static ref START_WEEK: DateTime<Utc> = Utc.ymd(2018, 9, 11).and_hms(17, 0, 0);
    }
    (now - *START_WEEK).num_weeks() % 3
}

fn protocol_week_number(now: BotDateTime) -> i64 {
    lazy_static! {
        static ref START_WEEK: DateTime<Utc> = Utc.ymd(2018, 5, 8).and_hms(17, 0, 0);
    }
    (now - *START_WEEK).num_weeks() % 5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_weeks() {
        // Week Oct 16-22 - Strongest Curse (2)
        assert_eq!(dc_week_number(Utc.ymd(2018, 10, 20).and_hms(12, 0, 0)), 2);
        // Week Oct 23-29 - Weak Curse (0)
        assert_eq!(dc_week_number(Utc.ymd(2018, 10, 24).and_hms(12, 0, 0)), 0);
        // Week Oct 30-Nov 5 - Growing Curse (1)
        assert_eq!(dc_week_number(Utc.ymd(2018, 11, 1).and_hms(12, 0, 0)), 1);
    }

    #[test]
    fn test_protocol_weeks() {
        assert_eq!(
            protocol_week_number(Utc.ymd(2018, 10, 20).and_hms(12, 0, 0)),
            3
        );
    }
}
