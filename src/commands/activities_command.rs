use crate::models::{Activity, ActivityShortcut, NewActivity};
use crate::{Bot, BotCommand, DbConnection};
use diesel::{self, prelude::*};
use diesel_derives_traits::NewModel;
use futures::Future;
use itertools::Itertools;
use std::collections::HashMap;

pub struct ActivitiesCommand;

command_ctor!(ActivitiesCommand);

impl ActivitiesCommand {
    fn usage(bot: &Bot, message: &telebot::objects::Message) {
        bot.send_plain_reply(
            &message,
            "Activities command help:

/activities
    Lists all available activities shortcuts

Admin-only mode:

/activities ids
    Lists IDs of all activities
/activities add KV
    Create new activity from KV pairs (see below)
/activities edit ID KV
    Modify activity with given ID by updating all given KVs
/activities addsc ID shortcut
    Add activity shortcut for activity ID

KV pairs are space-separated pairs of key=value elements
String arguments may be in quotes, but this is optional.

Supported KV pairs for add/edit commands:

name=activity name (e.g. Crucible)    <mandatory>
mode=activity mode (e.g. Iron Banner) <optional>
min_fireteam_size=n                   <mandatory>
max_fireteam_size=n                   <mandatory>
min_light=n                           <optional>
min_level=n                           <optional>
shortcut=sc                           <optional>"
                .into(),
        );
    }
}

impl BotCommand for ActivitiesCommand {
    fn prefix(&self) -> &'static str {
        "/activities"
    }

    fn description(&self) -> &'static str {
        "List available activity shortcuts"
    }

    fn execute(
        &self,
        bot: &Bot,
        message: &telebot::objects::Message,
        _command: Option<String>,
        args: Option<String>,
    ) {
        let connection = bot.connection();

        if args.is_none() {
            use schema::activities::dsl::{activities, id};
            use schema::activityshortcuts::dsl::{activityshortcuts, game, name};

            // Just /activities
            let games = activityshortcuts
                .select(game)
                .distinct()
                .order(game.asc())
                .load::<String>(&connection)
                .expect("Failed to load activity shortcuts");

            let mut text = "Activities: use a short name:\n".to_owned();

            for game_name in games {
                text += &format!("*** <b>{0}</b>:\n", game_name);
                let shortcuts = activityshortcuts
                    .filter(game.eq(game_name))
                    .order(name.asc())
                    .load::<ActivityShortcut>(&connection)
                    .expect("TEMP loading @FIXME");
                for shortcut in shortcuts {
                    let link_name = activities
                        .filter(id.eq(shortcut.link))
                        .first::<Activity>(&connection)
                        .expect("Failed to load activity");

                    text += &format!(
                        "<b>{name}</b>\t{link}\n",
                        name = shortcut.name,
                        link = link_name.format_name(),
                    );
                }
                text += "\n";
            }

            return bot.send_html_reply(&message, text);
        }

        let args = args.unwrap();
        let args: Vec<&str> = args.splitn(2, ' ').collect();

        if args.len() < 1 {
            return ActivitiesCommand::usage(bot, &message);
        }

        // @todo add admin check here

        match args[0] {
            "ids" => {
                use schema::activities::dsl::{activities, id, mode, name};

                let games = activities
                    .select((id, name, mode))
                    .order(id.asc())
                    .load::<(i32, String, Option<String>)>(&connection)
                    .expect("Failed to load activities");

                let mut text = "Activities:\n\n".to_string();
                for (id_, name_, mode_) in games {
                    let mode_ = if mode_.is_none() {
                        "".to_string()
                    } else {
                        mode_.unwrap()
                    };
                    text += &format!("{}. {} {}\n", id_, name_, mode_);
                }
                bot.send_plain_reply(&message, text);
            }
            "add" => {
                let argmap = parse_kv_args(args[1]);
                if argmap.is_none() {
                    return bot.send_plain_reply(
                        &message,
                        "Invalid activity specification, see help.".into(),
                    );
                }
                let mut argmap = argmap.unwrap();
                let name = argmap.remove("name");
                if name.is_none() {
                    return bot
                        .send_plain_reply(&message, "Must specify activity name, see help.".into());
                }

                let min_fireteam_size = argmap.remove("min_fireteam_size");
                if min_fireteam_size.is_none() {
                    return bot.send_plain_reply(
                        &message,
                        "Must specify min_fireteam_size, see help.".into(),
                    );
                }
                let min_fireteam_size = min_fireteam_size.unwrap().parse::<i32>();
                if min_fireteam_size.is_err() {
                    return bot
                        .send_plain_reply(&message, "min_fireteam_size must be a number".into());
                }
                let min_fireteam_size = min_fireteam_size.unwrap();

                let max_fireteam_size = argmap.remove("max_fireteam_size");
                if max_fireteam_size.is_none() {
                    return bot.send_plain_reply(
                        &message,
                        "Must specify max_fireteam_size, see help.".into(),
                    );
                }
                let max_fireteam_size = max_fireteam_size.unwrap().parse::<i32>();
                if max_fireteam_size.is_err() {
                    return bot
                        .send_plain_reply(&message, "max_fireteam_size must be a number".into());
                }
                let max_fireteam_size = max_fireteam_size.unwrap();

                // check no duplicates -- ?
                let mut act = NewActivity {
                    name: name.unwrap().into(),
                    mode: None,
                    min_fireteam_size,
                    max_fireteam_size,
                    min_level: None,
                    min_light: None,
                };

                for (key, val) in argmap {
                    match key {
                        "min_light" => {
                            let val = val.parse::<i32>();
                            if val.is_err() {
                                return bot.send_plain_reply(
                                    &message,
                                    "min_light must be a number".into(),
                                );
                            }
                            act.min_light = Some(val.unwrap())
                        }
                        "min_level" => {
                            let val = val.parse::<i32>();
                            if val.is_err() {
                                return bot.send_plain_reply(
                                    &message,
                                    "min_level must be a number".into(),
                                );
                            }
                            act.min_level = Some(val.unwrap())
                        }
                        "mode" => act.mode = Some(val.into()),
                        "shortcut" => { /* ignore here */ }
                        _ => {
                            return bot
                                .send_plain_reply(&message, format!("Unknown field name {}", key));
                        }
                    }
                }

                match act.save(&connection) {
                    Ok(act) => bot.send_plain_reply(&message, format!("Activity {} added.", act.format_name())),
                    Err(e) => bot.send_plain_reply(&message, format!("Error creating activity. {:?}", e)),
                }
            }
            "addsc" => {
                bot.send_plain_reply(&message, "ADD SC".into());
            }
            "edit" => {
                bot.send_plain_reply(&message, "EDIT".into());
            }
            "delete" => {
                // delete activity by id, and all its shortcuts
                // if there are plannedactivities with this id, cannot be deleted - check FOREIGN KEY constraints
            }
            _ => {
                bot.send_plain_reply(&message, "Unknown activities operation".into());
            }
        }
    }
}

fn parse_kv_args(args: &str) -> Option<HashMap<&str, &str>> {
    fn final_collect(args: Vec<&str>) -> HashMap<&str, &str> {
        return args
            .into_iter()
            .tuples()
            .map(|(k, v)| (k, v.trim_matches('"')))
            .collect::<HashMap<_, _>>();
    }

    let fragments: Vec<&str> = args.split('=').collect();

    trace!("{:?}", fragments);

    if fragments.len() < 2 {
        None
    } else if fragments.len() == 2 {
        // only single parameter
        Some(final_collect(fragments))
    } else {
        // ['max_fireteam_size', '1', 'name', '6', 'mode', '"Last Wish, Enhance"']
        let subfrags = itertools::Itertools::flatten(fragments[1..fragments.len() - 1].iter().map(
            |x: &&str| {
                x.rsplitn(2, ' ')
                    .collect::<Vec<&str>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<&str>>()
            },
        )).collect::<Vec<&str>>();

        trace!("{:?}", subfrags);

        let mut final_ = vec![fragments[0]];
        final_.extend(subfrags);
        final_.extend(vec![fragments[fragments.len() - 1]]);

        trace!("Final {:?}", final_);

        let the_map = final_collect(final_);

        trace!(".. as map {:?}", the_map);

        Some(the_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_algorithm() {
        // min_fireteam_size=1 max_fireteam_size=6 name="Last Wish, Enhanced" mode="prestige"
        let args =
            r#"min_fireteam_size=1 max_fireteam_size=6 name="Last Wish, Enhanced" mode="prestige""#;
        let result = parse_kv_args(args);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.len(), 4);

        let args = r#"name="Last Wish, Enhanced""#;
        let result = parse_kv_args(args);
        assert!(result.is_some());
        let mut result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.remove("name"), Some("Last Wish, Enhanced"));

        let args = r#"whatever else"#;
        let result = parse_kv_args(args);
        assert!(result.is_none());
    }
}
