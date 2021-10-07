use {
    crate::{
        commands::admin_check,
        models::{Activity, ActivityShortcut, NewActivity, NewActivityShortcut},
        BotCommand, BotMenu, DbConnection, UpdateMessage,
    },
    diesel::{self, prelude::*},
    diesel_derives_traits::{Model, NewModel},
    futures::Future,
    itertools::Itertools,
    std::collections::HashMap,
    teloxide::prelude::*,
};

#[derive(Clone)]
pub struct ActivitiesCommand;

command_ctor!(ActivitiesCommand);

impl ActivitiesCommand {
    fn usage(bot: &BotMenu, message: &UpdateMessage) {
        bot.send_plain_reply(
            &message,
            "Activities command help:

/activities
    Lists all available activities shortcuts.

Admin-only mode:

/activities ids
    Lists IDs of all activities.
/activities add KV
    Create new activity from KV pairs (see below).
/activities edit ID KV
    Modify activity with given ID by updating all given KVs.
/activities addsc ID shortcut <Game Name>
    Add activity shortcut for activity ID.
/activities delete ID
    Remove activity if it doesn't have any activities planned.

KV pairs are space-separated pairs of key=value elements
String arguments may be in quotes, but this is optional.

Supported KV pairs for add/edit commands:

name=activity name (e.g. Crucible)    <mandatory>
mode=activity mode (e.g. Iron Banner) <optional>
min_fireteam_size=n                   <mandatory>
max_fireteam_size=n                   <mandatory>
min_light=n                           <optional>
min_level=n                           <optional>"
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
        bot: &BotMenu,
        message: &UpdateWithCx<AutoSend<Bot>, Message>,
        _command: Option<String>,
        args: Option<String>,
    ) {
        let connection = bot.connection();

        if args.is_none() {
            use crate::schema::{
                activities::dsl::{activities, id},
                activityshortcuts::dsl::{activityshortcuts, game, name},
            };

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

        if args.is_empty() {
            return ActivitiesCommand::usage(bot, &message);
        }

        let admin = admin_check(bot, message, &connection);
        if admin.is_none() {
            return bot.send_plain_reply(&message, "You are not admin".to_string());
        }

        match args[0] {
            "ids" => {
                use crate::schema::activities::dsl::{activities, id, mode, name};

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
                if args.len() < 2 {
                    bot.send_plain_reply(&message, "Syntax: /activities add KV".into());
                    return ActivitiesCommand::usage(bot, &message);
                }

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
                    return bot.send_plain_reply(
                        &message,
                        "Must specify activity name, see help.".into(),
                    );
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
                        _ => {
                            return bot
                                .send_plain_reply(&message, format!("Unknown field name {}", key));
                        }
                    }
                }

                match act.save(&connection) {
                    Ok(act) => bot.send_plain_reply(
                        &message,
                        format!("Activity {} added.", act.format_name()),
                    ),
                    Err(e) => {
                        bot.send_plain_reply(&message, format!("Error creating activity. {:?}", e))
                    }
                }
            }
            "addsc" => {
                if args.len() < 2 {
                    return bot.send_plain_reply(
                        &message,
                        "Syntax: /activities addsc ActivityID ShortcutName Game name".into(),
                    );
                }

                let args: Vec<&str> = args[1].splitn(3, ' ').collect();
                if args.len() != 3 {
                    return bot.send_plain_reply(
                        &message,
                        "To add a shortcut specify activity ID, shortcut name and then game name"
                            .into(),
                    );
                }

                let link = args[0].parse::<i32>();
                if link.is_err() {
                    return bot.send_plain_reply(&message, "ActivityID must be a number".into());
                }
                let link = link.unwrap();
                let name = args[1].to_string();
                let game = args[2].to_string();

                let act = Activity::find_one(&connection, &link).expect("Failed to run SQL");

                if act.is_none() {
                    return bot
                        .send_plain_reply(&message, format!("Activity {} was not found.", link));
                }

                let shortcut = NewActivityShortcut { name, game, link };

                if shortcut.save(&connection).is_err() {
                    bot.send_plain_reply(&message, "Error creating shortcut".into());
                }

                bot.send_plain_reply(&message, "Shortcut added".into());
            }
            "edit" => {
                if args.len() < 2 {
                    bot.send_plain_reply(&message, "Syntax: /activities edit ID KV".into());
                    return ActivitiesCommand::usage(bot, &message);
                }

                let args: Vec<&str> = args[1].splitn(2, ' ').collect();
                if args.len() != 2 {
                    return bot.send_plain_reply(
                        &message,
                        "To edit specify activity id and then KV pairs".into(),
                    );
                }

                let id = args[0].parse::<i32>();
                if id.is_err() {
                    return bot.send_plain_reply(&message, "ActivityID must be a number".into());
                }
                let id = id.unwrap();

                let act = Activity::find_one(&connection, &id).expect("Failed to run SQL");

                if act.is_none() {
                    return bot
                        .send_plain_reply(&message, format!("Activity {} was not found.", id));
                }
                let mut act = act.unwrap();

                let argmap = parse_kv_args(args[1]);
                if argmap.is_none() {
                    return bot.send_plain_reply(
                        &message,
                        "Invalid activity specification, see help.".into(),
                    );
                }
                let argmap = argmap.unwrap();

                for (key, val) in argmap {
                    match key {
                        "name" => act.name = val.into(),
                        "min_fireteam_size" => {
                            let val = val.parse::<i32>();
                            if val.is_err() {
                                return bot.send_plain_reply(
                                    &message,
                                    "min_fireteam_size must be a number".into(),
                                );
                            }
                            act.min_fireteam_size = val.unwrap()
                        }
                        "max_fireteam_size" => {
                            let val = val.parse::<i32>();
                            if val.is_err() {
                                return bot.send_plain_reply(
                                    &message,
                                    "max_fireteam_size must be a number".into(),
                                );
                            }
                            act.max_fireteam_size = val.unwrap()
                        }
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
                        _ => {
                            return bot
                                .send_plain_reply(&message, format!("Unknown field name {}", key));
                        }
                    }
                }

                match act.save(&connection) {
                    Ok(act) => bot.send_plain_reply(
                        &message,
                        format!("Activity {} updated.", act.format_name()),
                    ),
                    Err(e) => {
                        bot.send_plain_reply(&message, format!("Error updating activity. {:?}", e))
                    }
                }
            }
            "delete" => {
                if args.len() < 2 {
                    bot.send_plain_reply(&message, "Syntax: /activities delete ID".into());
                    return ActivitiesCommand::usage(bot, &message);
                }

                let id = args[1].parse::<i32>();
                if id.is_err() {
                    return bot.send_plain_reply(&message, "ActivityID must be a number".into());
                }
                let id = id.unwrap();

                let act = Activity::find_one(&connection, &id).expect("Failed to run SQL");

                if act.is_none() {
                    return bot
                        .send_plain_reply(&message, format!("Activity {} was not found.", id));
                }

                let act = act.unwrap();

                let name = act.format_name();

                match act.destroy(&connection) {
                    Ok(_) => bot.send_plain_reply(&message, format!("Activity {} deleted.", name)),
                    Err(e) => {
                        bot.send_plain_reply(&message, format!("Error deleting activity. {:?}", e))
                    }
                }
            }
            _ => {
                bot.send_plain_reply(&message, "Unknown activities operation".into());
                ActivitiesCommand::usage(bot, &message);
            }
        }
    }
}

fn parse_kv_args(args: &str) -> Option<HashMap<&str, &str>> {
    fn final_collect(args: Vec<&str>) -> HashMap<&str, &str> {
        args.into_iter()
            .tuples()
            .map(|(k, v)| (k, v.trim_matches('"')))
            .collect::<HashMap<_, _>>()
    }

    let fragments: Vec<&str> = args.split('=').collect();

    log::trace!("{:?}", fragments);

    if fragments.len() < 2 {
        None
    } else if fragments.len() == 2 {
        // only single parameter
        Some(final_collect(fragments))
    } else {
        // ['max_fireteam_size', '1', 'name', '6', 'mode', '"Last Wish, Enhance"']
        let subfrags = fragments[1..fragments.len() - 1]
            .iter()
            .map(|x: &&str| {
                x.rsplitn(2, ' ')
                    .collect::<Vec<&str>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<&str>>()
            })
            .flatten()
            .collect::<Vec<&str>>();

        log::trace!("{:?}", subfrags);

        let mut final_ = vec![fragments[0]];
        final_.extend(subfrags);
        final_.extend(vec![fragments[fragments.len() - 1]]);

        log::trace!("Final {:?}", final_);

        let the_map = final_collect(final_);

        log::trace!(".. as map {:?}", the_map);

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
