use chrono::NaiveDateTime;
use crate::commands::send_html_message;
use crate::DbConnection;
use diesel::*;
use diesel_derives_traits::NewModel;
use failure::Error;
use futures::Future;
use models::{Alert, NewAlert};
use rss::{Channel, Guid};
use schema::alerts::dsl::*;
use telebot::{functions::*, RcBot};

const RSS_DATE_FORMAT: &'static str = "%a, %d %b %Y %H:%M:%S %z"; // Thu, 10 May 2018 12:08:20 +0000

pub fn check(
    bot: &RcBot,
    chat_id: telebot::objects::Integer,
    connection: &DbConnection,
) -> Result<(), Error> {
    let channel = Channel::from_url("http://content.ps4.warframe.com/dynamic/rss.php")?;
    let mut alert_list = vec![];
    for item in channel.into_items() {
        println!("{:?}", item); //@todo remove

        let def_guid = &Guid::default();
        let guid_value = item.guid().unwrap_or(def_guid).value();
        let existing_alert_count = alerts
            .filter(guid.eq(guid_value))
            .count()
            .first::<i64>(connection);

        if existing_alert_count == Ok(0) {
            let alert = NewAlert {
                guid: guid_value,
                title: item.title().unwrap_or(""),
                alert_type: item.author(),
                start_date: NaiveDateTime::parse_from_str(
                    item.pub_date().unwrap_or(""),
                    RSS_DATE_FORMAT,
                ).map(|v| Some(v))
                .unwrap_or(None),
                expiry_date: NaiveDateTime::parse_from_str(
                    item.extensions()
                        .get("wf")
                        .and_then(|ext| ext.get("wf:expiry"))
                        .map(|v| v[0].value().unwrap_or(""))
                        .unwrap_or(""),
                    RSS_DATE_FORMAT,
                ).map(|v| Some(v))
                .unwrap_or(None),
                faction: item
                    .extensions()
                    .get("wf")
                    .and_then(|ext| ext.get("wf:faction"))
                    .map(|v| v[0].value().unwrap_or("")),
                flavor: item.description(),
            };

            alert_list.push(alert.save(connection)?);
        }
    }

    alert_list.sort_by_key(|x| x.expiry_date);

    // Publish all new alerts
    for item in alert_list.iter().filter(|x| x.alert_type == "Alert") {
        println!("{}", item);
        send_html_message(bot, chat_id, format!("{}", item));
    }

    Ok(())
}
