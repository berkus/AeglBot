use crate::bot_actor::{Format, Notify};
use {
    crate::{
        bot_actor::{BotActorMsg, SendMessage},
        datetime::{reference_date, BotDateTime},
        DbConnection,
    },
    anyhow::Result,
    chrono::{DateTime, Duration, TimeZone, Utc},
    futures::Future,
    riker::{actor::Tell, actors::ActorRef},
};
// use plurals::{Lang, Plural};

// Destiny schedules on weekly featured Raid:

pub fn raid_cycle() -> String {
    let raids: [&'static str; 4] = [
        "*Crota's End*, activity id: *crw*",
        "*Vault of Glass*, activity id: *vogw*",
        "*King's Fall*, activity id: *kfw*",
        "*Wrath of the Machine*, activity id: *wotmw*",
    ];

    let raid_week = raid_week_number(reference_date()) as usize;
    let next_week = raid_week_number(reference_date() + Duration::weeks(1)) as usize;
    format!(
        "Weekly Featured Raid: {}\nNext Week Raid: {}",
        raids[raid_week], raids[next_week]
    )
}

fn raid_week_number(now: BotDateTime) -> i64 {
    lazy_static! {
        static ref START_WEEK: DateTime<Utc> = Utc.ymd(2019, 12, 31).and_hms(17, 0, 0);
    }
    (now - *START_WEEK).num_weeks() % 4
}

// Destiny 2 schedules on tracking:

// const WEEKS: Lang = Lang::En {
//     singular: "week",
//     plural: "weeks",
// };

// 1. Daily resets at 20:00 MSK (17:00 UTC) every day
pub fn daily_reset(bot: ActorRef<BotActorMsg>, chat_id: teloxide::types::ChatId) -> Result<()> {
    bot.tell(
        SendMessage("⚡️ Daily reset".into(), chat_id, Format::Plain, Notify::Off),
        None,
    );
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
        "💫 Dreaming City: {} \\- [Ascendant Chests]({})\n\\(Shattered Throne is always available\\)",
        curses[dc_week], urls[dc_week]
    )
}

pub fn ascendant_challenge_cycle() -> String {
    let challenges: [&'static str; 6] = [
        "Agonarch Abyss",
        "Cimmerian Garrison",
        "Ouroborea",
        "Forfeit Shrine",
        "Shattered Ruins",
        "Keep of Honed Edges",
    ];
    let locations: [&'static str; 6] = [
        "Bay of Drowned Wishes",
        "Chamber of Starlight",
        "Aphelion's Rest",
        "Gardens of Esila",
        "Spine of Keres",
        "Harbinger's Seclude",
    ];
    let urls: [&'static str; 6] = [
        "https://www.youtube.com/watch?v=xvHovs5CUMI",
        "https://www.youtube.com/watch?v=sAZrihxy_30",
        "https://www.youtube.com/watch?v=9-1a_jUxckQ",
        "https://www.youtube.com/watch?v=29Z0kg8MFQY",
        "https://www.youtube.com/watch?v=XUWD-IVuoHg",
        "https://www.youtube.com/watch?v=cDaNB-GEP-o",
    ];

    let ac_week = ascendant_challenge_week_number(reference_date()) as usize;
    format!(
        "[Ascendant Challenge](https://www.shacknews.com/article/109219/ascendant-challenge-schedule-and-location-destiny-2): [{name}]({url}) in the {loc}",
        name = challenges[ac_week],
        loc = locations[ac_week],
        url = urls[ac_week],
    )
}

// 2. Weekly (main) resets at 20:00 msk every Tue
// 6. On main reset: change in Dreaming City curse
//    dreaming city on 3-week schedule
// 7. On main reset: change in Dreaming City Ascendant Challenges
//    dreaming city challenges on 6-week schedule
pub fn major_weekly_reset(
    bot: ActorRef<BotActorMsg>,
    chat_id: teloxide::types::ChatId,
) -> Result<()> {
    let msg = format!(
        "⚡️ Weekly reset:\n\n{d1week}\n\n{d2week}",
        d1week = this_week_in_d1(),
        d2week = this_week_in_d2(),
    );
    bot.tell(
        SendMessage(msg, chat_id, Format::Markdown, Notify::Off),
        None,
    );
    Ok(())
}

pub fn this_week_in_d1() -> String {
    format!("This week in Destiny 1:\n\n{}", raid_cycle())
}

pub fn this_week_in_d2() -> String {
    format!(
        "This week in Destiny 2:\n\n{}\n{}",
        dreaming_city_cycle(),
        ascendant_challenge_cycle()
    )
}

fn dc_week_number(now: BotDateTime) -> i64 {
    lazy_static! {
        static ref START_WEEK: DateTime<Utc> = Utc.ymd(2018, 9, 11).and_hms(17, 0, 0);
    }
    (now - *START_WEEK).num_weeks() % 3
}

fn ascendant_challenge_week_number(now: BotDateTime) -> i64 {
    lazy_static! {
        static ref START_WEEK: DateTime<Utc> = Utc.ymd(2021, 7, 6).and_hms(17, 0, 0);
    }
    (now - *START_WEEK).num_weeks() % 6
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

    #[test]
    fn test_raid_weeks() {
        assert_eq!(raid_week_number(Utc.ymd(2020, 1, 28).and_hms(21, 0, 0)), 0);
        assert_eq!(raid_week_number(Utc.ymd(2020, 1, 27).and_hms(12, 0, 0)), 3);
    }
}
