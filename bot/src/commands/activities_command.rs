use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{admin_check, match_command},
        models::{Activity, ActivityShortcut, NewActivity, NewActivityShortcut},
        BotCommand,
    },
    diesel::{self, prelude::*},
    diesel_derives_traits::{Model, NewModel},
    itertools::Itertools,
    riker::actors::Tell,
    std::collections::HashMap,
};

command_actor!(ActivitiesCommand, [ActorUpdateMessage]);

impl ActivitiesCommand {
    fn send_reply<S>(&self, message: &ActorUpdateMessage, reply: S)
    where
        S: Into<String>,
    {
        self.bot_ref.tell(
            SendMessageReply(reply.into(), message.clone(), Format::Plain, Notify::Off),
            None,
        );
    }

    fn usage(&self, message: &ActorUpdateMessage) {
        self.send_reply(
            message,
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
min_level=n                           <optional>",
        );
    }
}

impl BotCommand for ActivitiesCommand {
    fn prefix() -> &'static str {
        "/activities"
    }

    fn description() -> &'static str {
        "List available activity shortcuts"
    }
}

// Need to find a way to partially implement the Actor trait here, esp to set up sub-command actors
// impl Actor for ActivitiesCommand {
//     // Create subcommand actors somewhere here...
//
//     fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
//         todo!()
//     }
//
//     fn post_start(&mut self, ctx: &Context<Self::Msg>) {
//         todo!()
//     }
// }

impl Receive<ActorUpdateMessage> for ActivitiesCommand {
    type Msg = ActivitiesCommandMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, message: ActorUpdateMessage, _sender: Sender) {
        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            let connection = self.connection();

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

                return self.bot_ref.tell(
                    SendMessageReply(text, message, Format::Html, Notify::Off),
                    None,
                );
            }

            // some args - pass to a subcommand

            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(2, ' ').collect();

            if args.is_empty() {
                return self.usage(&message);
            }

            let admin = admin_check(&self.bot_ref, &message, &connection);
            if admin.is_none() {
                return self.send_reply(&message, "You are not admin");
            }

            // split into subcommands:
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
                        text += &format!("{}. {} {}\n", id_, name_, mode_.unwrap_or("".into()));
                    }
                    self.send_reply(&message, text);
                }
                "add" => {
                    if args.len() < 2 {
                        self.send_reply(&message, "Syntax: /activities add KV");
                        return self.usage(&message);
                    }

                    let argmap = parse_kv_args(args[1]);
                    if argmap.is_none() {
                        return self
                            .send_reply(&message, "Invalid activity specification, see help.");
                    }
                    let mut argmap = argmap.unwrap();
                    let name = argmap.remove("name");
                    if name.is_none() {
                        return self.send_reply(&message, "Must specify activity name, see help.");
                    }

                    let min_fireteam_size = argmap.remove("min_fireteam_size");
                    if min_fireteam_size.is_none() {
                        return self
                            .send_reply(&message, "Must specify min_fireteam_size, see help.");
                    }
                    let min_fireteam_size = min_fireteam_size.unwrap().parse::<i32>();
                    if min_fireteam_size.is_err() {
                        return self.send_reply(&message, "min_fireteam_size must be a number");
                    }
                    let min_fireteam_size = min_fireteam_size.unwrap();

                    let max_fireteam_size = argmap.remove("max_fireteam_size");
                    if max_fireteam_size.is_none() {
                        return self
                            .send_reply(&message, "Must specify max_fireteam_size, see help.");
                    }
                    let max_fireteam_size = max_fireteam_size.unwrap().parse::<i32>();
                    if max_fireteam_size.is_err() {
                        return self.send_reply(&message, "max_fireteam_size must be a number");
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
                                    return self.send_reply(&message, "min_light must be a number");
                                }
                                act.min_light = Some(val.unwrap())
                            }
                            "min_level" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.send_reply(&message, "min_level must be a number");
                                }
                                act.min_level = Some(val.unwrap())
                            }
                            "mode" => act.mode = Some(val.into()),
                            _ => {
                                return self
                                    .send_reply(&message, format!("Unknown field name {}", key));
                            }
                        }
                    }

                    match act.save(&connection) {
                        Ok(act) => self
                            .send_reply(&message, format!("Activity {} added.", act.format_name())),
                        Err(e) => {
                            self.send_reply(&message, format!("Error creating activity. {:?}", e))
                        }
                    }
                }
                "addsc" => {
                    if args.len() < 2 {
                        return self.send_reply(
                            &message,
                            "Syntax: /activities addsc ActivityID ShortcutName Game name",
                        );
                    }

                    let args: Vec<&str> = args[1].splitn(3, ' ').collect();
                    if args.len() != 3 {
                        return self.send_reply(
                            &message,
                            "To add a shortcut specify activity ID, shortcut name and then the game name",
                        );
                    }

                    let link = args[0].parse::<i32>();
                    if link.is_err() {
                        return self.send_reply(&message, "ActivityID must be a number");
                    }
                    let link = link.unwrap();
                    let name = args[1].to_string();
                    let game = args[2].to_string();

                    let act = Activity::find_one(&connection, &link).expect("Failed to run SQL");

                    if act.is_none() {
                        return self
                            .send_reply(&message, format!("Activity {} was not found.", link));
                    }

                    let shortcut = NewActivityShortcut { name, game, link };

                    if shortcut.save(&connection).is_err() {
                        return self.send_reply(&message, "Error creating shortcut");
                    }

                    self.send_reply(&message, "Shortcut added");
                }
                "edit" => {
                    if args.len() < 2 {
                        self.send_reply(&message, "Syntax: /activities edit ID KV");
                        return self.usage(&message);
                    }

                    let args: Vec<&str> = args[1].splitn(2, ' ').collect();
                    if args.len() != 2 {
                        return self.send_reply(
                            &message,
                            "To edit first specify Activity ID and then key=value pairs",
                        );
                    }

                    let id = args[0].parse::<i32>();
                    if id.is_err() {
                        return self.send_reply(&message, "ActivityID must be a number");
                    }
                    let id = id.unwrap();

                    let act = Activity::find_one(&connection, &id).expect("Failed to run SQL");

                    if act.is_none() {
                        return self
                            .send_reply(&message, format!("Activity {} was not found.", id));
                    }
                    let mut act = act.unwrap();

                    let argmap = parse_kv_args(args[1]);
                    if argmap.is_none() {
                        return self
                            .send_reply(&message, "Invalid activity specification, see help.");
                    }
                    let argmap = argmap.unwrap();

                    for (key, val) in argmap {
                        match key {
                            "name" => act.name = val.into(),
                            "min_fireteam_size" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.send_reply(
                                        &message,
                                        "min_fireteam_size must be a number",
                                    );
                                }
                                act.min_fireteam_size = val.unwrap()
                            }
                            "max_fireteam_size" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.send_reply(
                                        &message,
                                        "max_fireteam_size must be a number",
                                    );
                                }
                                act.max_fireteam_size = val.unwrap()
                            }
                            "min_light" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.send_reply(&message, "min_light must be a number");
                                }
                                act.min_light = Some(val.unwrap())
                            }
                            "min_level" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.send_reply(&message, "min_level must be a number");
                                }
                                act.min_level = Some(val.unwrap())
                            }
                            "mode" => act.mode = Some(val.into()),
                            _ => {
                                return self
                                    .send_reply(&message, format!("Unknown field name {}", key));
                            }
                        }
                    }

                    match act.save(&connection) {
                        Ok(act) => self.send_reply(
                            &message,
                            format!("Activity {} updated.", act.format_name()),
                        ),
                        Err(e) => {
                            self.send_reply(&message, format!("Error updating activity. {:?}", e))
                        }
                    }
                }
                "delete" => {
                    if args.len() < 2 {
                        self.send_reply(&message, "Syntax: /activities delete ID");
                        return self.usage(&message);
                    }

                    let id = args[1].parse::<i32>();
                    if id.is_err() {
                        return self.send_reply(&message, "ActivityID must be a number");
                    }
                    let id = id.unwrap();

                    let act = Activity::find_one(&connection, &id).expect("Failed to run SQL");

                    if act.is_none() {
                        return self
                            .send_reply(&message, format!("Activity {} was not found.", id));
                    }

                    let act = act.unwrap();

                    let name = act.format_name();

                    match act.destroy(&connection) {
                        Ok(_) => self.send_reply(&message, format!("Activity {} deleted.", name)),
                        Err(e) => {
                            self.send_reply(&message, format!("Error deleting activity. {:?}", e))
                        }
                    }
                }
                _ => {
                    self.send_reply(&message, "Unknown activities operation");
                    self.usage(&message);
                }
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

    match fragments.len() {
        x if x < 2 => None,
        2 =>         // only single parameter
            Some(final_collect(fragments)),
        _ => {
            // ['max_fireteam_size', '1', 'name', '6', 'mode', '"Last Wish, Enhance"']
            let subfrags = fragments[1..fragments.len() - 1]
                .iter()
                .flat_map(|x: &&str| {
                    x.rsplitn(2, ' ')
                        .collect::<Vec<&str>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<&str>>()
                })
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
