use chrono::{prelude::*, DateTime, Duration, Utc};
use crate::{Bot, DbConnection};
use diesel::prelude::*;
use diesel_derives_traits::NewModel;
use failure::Error;
use futures::Future;
use models::{Alert, NewAlert};
use rss::{Channel, Guid};
use schema::alerts::dsl::*;
use telebot::{functions::*, RcBot};

const RSS_DATE_FORMAT: &str = "%a, %d %b %Y %H:%M:%S %z"; // Thu, 10 May 2018 12:08:20 +0000

fn new_alert_from_rss_item(item: rss::Item) -> NewAlert {
    let def_guid = &Guid::default();
    let guid_value = item.guid().unwrap_or(def_guid).value();
    NewAlert {
        guid: guid_value.into(),
        title: item.title().unwrap_or("").into(),
        kind: item.author().map(|x| x.into()),
        start_date: DateTime::parse_from_str(item.pub_date().unwrap_or(""), RSS_DATE_FORMAT)
            .map(|v| Some(v.with_timezone(&Utc)))
            .unwrap_or(None),
        expiry_date: DateTime::parse_from_str(
            item.extensions()
                .get("wf")
                .and_then(|ext| ext.get("expiry"))
                .map(|v| v[0].value().unwrap_or(""))
                .unwrap_or(""),
            RSS_DATE_FORMAT,
        ).map(|v| Some(v.with_timezone(&Utc)))
        .unwrap_or(None),
        faction: item
            .extensions()
            .get("wf")
            .and_then(|ext| ext.get("faction"))
            .map(|v| v[0].value().unwrap_or(""))
            .map(|x| x.into()),
        flavor: item.description().map(|x| x.into()),
    }
}

pub fn check(bot: &Bot, chat_id: telebot::objects::Integer) -> Result<(), Error> {
    info!("alerts check");
    let connection = bot.connection();
    let channel = Channel::from_url("http://content.ps4.warframe.com/dynamic/rss.php")?;
    let mut alert_list = vec![];
    for item in channel.into_items() {
        trace!("{:?}", item);

        let def_guid = &Guid::default();
        let guid_value = item.guid().unwrap_or(def_guid).value();
        let existing_alert_count = alerts
            .filter(guid.eq(guid_value))
            .count()
            .first::<i64>(&connection);

        if existing_alert_count == Ok(0) {
            let alert = new_alert_from_rss_item(item);
            alert_list.push(alert.save(&connection)?);
        }
    }

    alert_list.sort_by_key(|x| x.expiry_date);

    // Publish all new alerts
    for item in alert_list.iter() {
        info!("{}", item);
        if item.is_important() {
            bot.send_html_message_with_notification(
                chat_id,
                format!("⚠️ Important ⚠️\n\n{}", item),
            );
        } else {
            bot.send_html_message(chat_id, format!("{}", item));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static RSS_DATA: &str = r#"<?xml version="1.0" encoding="ISO-8859-1"?>
    <rss version="2.0" xmlns:wf="http://warframe.com/rss/v1">
    <channel>
    <title>Warframe PS4</title>
    <link>http://warframe.com</link>
    <description>Notifications</description>
    <language>en-us</language>
    <copyright>(C)2018 Warframe</copyright>
    <ttl>1</ttl>
    <item>
    <guid>5bb8abf52534495de77a8d2f</guid>
    <title>150 ENDO - 11000cr - Caliban (Uranus) - 53m</title>
    <author>Alert</author>
    <description>Protect Sensitive Data</description>
    <pubDate>Sat, 06 Oct 2018 12:39:28 +0000</pubDate>
    <wf:faction>FC_GRINEER</wf:faction>
    <wf:expiry>Sat, 06 Oct 2018 13:32:08 +0000</wf:expiry>
    </item>
    <item>
    <guid>5bb8a7faae5a28cb8250c5b7</guid>
    <title>Morphics (Resource) - 14900cr - Xini (Eris) - 36m</title>
    <author>Alert</author>
    <description>Detained Refugee Located</description>
    <pubDate>Sat, 06 Oct 2018 12:22:15 +0000</pubDate>
    <wf:faction>FC_CORPUS</wf:faction>
    <wf:expiry>Sat, 06 Oct 2018 12:58:08 +0000</wf:expiry>
    </item>
    <item>
    <guid>5bb8a34a04edc726e8f4b52a</guid>
    <title>80 ENDO - 3500cr - Lith (Earth) - 46m</title>
    <author>Alert</author>
    <description>Defeat Enemy Defense Forces</description>
    <pubDate>Sat, 06 Oct 2018 12:02:38 +0000</pubDate>
    <wf:faction>FC_INFESTATION</wf:faction>
    <wf:expiry>Sat, 06 Oct 2018 12:48:28 +0000</wf:expiry>
    </item>
    <item>
    <guid>5bb6de3ebb8e4ad21812fbfb</guid>
    <author>Invasion</author>
    <title>Corpus (Snipetron Vandal Blueprint) VS. Grineer (Latron Wraith Stock) - Pallas (Ceres)</title>
    <pubDate>Fri, 05 Oct 2018 03:56:11 +0000</pubDate>
    </item>
    <item>
    <guid>5bb6de3ebb8e4ad21812fbfc</guid>
    <author>Invasion</author>
    <title>Corpus (Dera Vandal Barrel) VS. Grineer (Latron Wraith Blueprint) - Cinxia (Ceres)</title>
    <pubDate>Fri, 05 Oct 2018 04:02:01 +0000</pubDate>
    </item>
    <item>
    <guid>5bb86a563c85eaf8b707c524</guid>
    <author>Outbreak</author>
    <title>3x Detonite Injector - Ares (Mars)</title>
    <pubDate>Sat, 06 Oct 2018 09:22:13 +0000</pubDate>
    </item>
    </channel>
    </rss>"#;

    #[test]
    fn test_rss_parse() {
        let channel = Channel::read_from(RSS_DATA.as_bytes()).unwrap();
        let mut it = channel.into_items().into_iter();
        {
            let item = it.next();
            assert!(item.is_some());
            let parsed = new_alert_from_rss_item(item.unwrap());
            assert_eq!(parsed.guid, "5bb8abf52534495de77a8d2f");
            assert_eq!(parsed.title, "150 ENDO - 11000cr - Caliban (Uranus) - 53m");
            assert_eq!(parsed.kind, Some("Alert".into()));
            assert_eq!(parsed.flavor, Some("Protect Sensitive Data".into()));
            assert_eq!(parsed.faction, Some("FC_GRINEER".into()));
            assert_eq!(
                parsed.start_date,
                Some(Utc.ymd(2018, 10, 6).and_hms(12, 39, 28))
            );
            assert_eq!(
                parsed.expiry_date,
                Some(Utc.ymd(2018, 10, 6).and_hms(13, 32, 8))
            );
        }
        {
            let item = it.next();
            assert!(item.is_some());
            let parsed = new_alert_from_rss_item(item.unwrap());
            assert_eq!(parsed.guid, "5bb8a7faae5a28cb8250c5b7");
            assert_eq!(
                parsed.title,
                "Morphics (Resource) - 14900cr - Xini (Eris) - 36m"
            );
            //            assert!(parsed.is_resource());
            assert_eq!(parsed.kind, Some("Alert".into()));
            assert_eq!(parsed.flavor, Some("Detained Refugee Located".into()));
            assert_eq!(parsed.faction, Some("FC_CORPUS".into()));
            assert_eq!(
                parsed.start_date,
                Some(Utc.ymd(2018, 10, 6).and_hms(12, 22, 15))
            );
            assert_eq!(
                parsed.expiry_date,
                Some(Utc.ymd(2018, 10, 6).and_hms(12, 58, 8))
            );
        }
        {
            let item = it.next();
            assert!(item.is_some());
            let parsed = new_alert_from_rss_item(item.unwrap());
            assert_eq!(parsed.guid, "5bb8a34a04edc726e8f4b52a");
            assert_eq!(parsed.title, "80 ENDO - 3500cr - Lith (Earth) - 46m");
            //            assert!(parsed.is_resource());
            assert_eq!(parsed.kind, Some("Alert".into()));
            assert_eq!(parsed.flavor, Some("Defeat Enemy Defense Forces".into()));
            assert_eq!(parsed.faction, Some("FC_INFESTATION".into()));
            assert_eq!(
                parsed.start_date,
                Some(Utc.ymd(2018, 10, 6).and_hms(12, 2, 38))
            );
            assert_eq!(
                parsed.expiry_date,
                Some(Utc.ymd(2018, 10, 6).and_hms(12, 48, 28))
            );
        }
        {
            let item = it.next();
            assert!(item.is_some());
            let parsed = new_alert_from_rss_item(item.unwrap());
            assert_eq!(parsed.guid, "5bb6de3ebb8e4ad21812fbfb");
            assert_eq!(parsed.title, "Corpus (Snipetron Vandal Blueprint) VS. Grineer (Latron Wraith Stock) - Pallas (Ceres)");
            assert_eq!(parsed.kind, Some("Invasion".into()));
            assert!(parsed.flavor.is_none());
            assert!(parsed.faction.is_none());
            assert_eq!(
                parsed.start_date,
                Some(Utc.ymd(2018, 10, 5).and_hms(3, 56, 11))
            );
            assert!(parsed.expiry_date.is_none());
        }
        {
            let item = it.next();
            assert!(item.is_some());
            let parsed = new_alert_from_rss_item(item.unwrap());
            assert_eq!(parsed.guid, "5bb6de3ebb8e4ad21812fbfc");
            assert_eq!(parsed.title, "Corpus (Dera Vandal Barrel) VS. Grineer (Latron Wraith Blueprint) - Cinxia (Ceres)");
            assert_eq!(parsed.kind, Some("Invasion".into()));
            assert!(parsed.flavor.is_none());
            assert!(parsed.faction.is_none());
            assert_eq!(
                parsed.start_date,
                Some(Utc.ymd(2018, 10, 5).and_hms(4, 2, 1))
            );
            assert!(parsed.expiry_date.is_none());
        }
        {
            let item = it.next();
            assert!(item.is_some());
            let parsed = new_alert_from_rss_item(item.unwrap());
            assert_eq!(parsed.guid, "5bb86a563c85eaf8b707c524");
            assert_eq!(parsed.title, "3x Detonite Injector - Ares (Mars)");
            assert_eq!(parsed.kind, Some("Outbreak".into()));
            assert!(parsed.flavor.is_none());
            assert!(parsed.faction.is_none());
            assert_eq!(
                parsed.start_date,
                Some(Utc.ymd(2018, 10, 6).and_hms(9, 22, 13))
            );
            assert!(parsed.expiry_date.is_none());
        }
        {
            let item = it.next();
            assert!(item.is_none());
        }
    }
}
