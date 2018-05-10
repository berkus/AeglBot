use models::NewAlert;
use rss::Channel;
use telegram_bot;

pub fn check(_api: &telegram_bot::Api, _chat_id: telegram_bot::ChatId) {
    let channel = Channel::from_url("http://content.ps4.warframe.com/dynamic/rss.php").unwrap();
    let alerts = vec![];
    for item in channel.into_items() {
        println!("{:?}", item);

        // @todo skip duplicates:
        //                 val alert = Alert.find { Alerts.guid eq g }.singleOrNull()

        let alert = NewAlert {
            guid: item.guid(),
            title: item.title().unwrap_or(""),
            alert_type: item.author(),
            start_date: item.pub_date(),
            expiry_date: item.extensions()
                .get("wf")
                .and_then(|ext| ext.get("wf:expiry")),
            faction: item.extensions()
                .get("wf")
                .and_then(|ext| ext.get("wf:faction")),
            flavor: item.description(),
        };
    }
    //             // Publish all new alerts (@todo sorted by expiry date)
    //             for (item in items.filter {i -> i.type == "Alert"}) {
    //                 sendReplyMessage(absSender, chatId.toLong(), "✊ Alert: ${item.title}", true)
    //             }
}
