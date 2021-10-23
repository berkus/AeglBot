use {
    crate::{
        bot_actor::{ActorUpdateMessage, BotActor, Format, Notify, SendMessageReply},
        commands::{admin_check, match_command},
        models::{Activity, ActivityShortcut, NewActivity, NewActivityShortcut},
        BotCommand, DbConnection,
    },
    diesel::{self, prelude::*},
    diesel_derives_traits::{Model, NewModel},
    futures::Future,
    itertools::Itertools,
    riker::actors::Tell,
    std::collections::HashMap,
    teloxide::prelude::*,
};

command_actor!(ActivitiesCommand, [ActorUpdateMessage]);

impl ActivitiesCommand {
    fn usage(&self, message: &ActorUpdateMessage) {
        self.bot_ref.tell(
            SendMessageReply(
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
                message.clone(),
                Format::Plain,
                Notify::Off,
            ),
            None,
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
        if let (Some(_), args) = match_command(message.update.text(), self.prefix(), &self.bot_name)
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
                return self.bot_ref.tell(
                    SendMessageReply(
                        "You are not admin".into(),
                        message,
                        Format::Plain,
                        Notify::Off,
                    ),
                    None,
                );
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
                    self.bot_ref.tell(
                        SendMessageReply(text, message, Format::Plain, Notify::Off),
                        None,
                    );
                }
                "add" => {
                    if args.len() < 2 {
                        self.bot_ref.tell(
                            SendMessageReply(
                                "Syntax: /activities add KV".into(),
                                message.clone(),
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                        return self.usage(&message);
                    }

                    let argmap = parse_kv_args(args[1]);
                    if argmap.is_none() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "Invalid activity specification, see help.".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }
                    let mut argmap = argmap.unwrap();
                    let name = argmap.remove("name");
                    if name.is_none() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "Must specify activity name, see help.".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }

                    let min_fireteam_size = argmap.remove("min_fireteam_size");
                    if min_fireteam_size.is_none() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "Must specify min_fireteam_size, see help.".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }
                    let min_fireteam_size = min_fireteam_size.unwrap().parse::<i32>();
                    if min_fireteam_size.is_err() {
                        // return error(ctx, message, ....)
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "min_fireteam_size must be a number".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }
                    let min_fireteam_size = min_fireteam_size.unwrap();

                    let max_fireteam_size = argmap.remove("max_fireteam_size");
                    if max_fireteam_size.is_none() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "Must specify max_fireteam_size, see help.".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }
                    let max_fireteam_size = max_fireteam_size.unwrap().parse::<i32>();
                    if max_fireteam_size.is_err() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "max_fireteam_size must be a number".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
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
                                    return self.bot_ref.tell(
                                        SendMessageReply(
                                            "min_light must be a number".into(),
                                            message,
                                            Format::Plain,
                                            Notify::Off,
                                        ),
                                        None,
                                    );
                                }
                                act.min_light = Some(val.unwrap())
                            }
                            "min_level" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.bot_ref.tell(
                                        SendMessageReply(
                                            "min_level must be a number".into(),
                                            message,
                                            Format::Plain,
                                            Notify::Off,
                                        ),
                                        None,
                                    );
                                }
                                act.min_level = Some(val.unwrap())
                            }
                            "mode" => act.mode = Some(val.into()),
                            _ => {
                                return self.bot_ref.tell(
                                    SendMessageReply(
                                        format!("Unknown field name {}", key),
                                        message,
                                        Format::Plain,
                                        Notify::Off,
                                    ),
                                    None,
                                );
                            }
                        }
                    }

                    match act.save(&connection) {
                        Ok(act) => self.bot_ref.tell(
                            SendMessageReply(
                                format!("Activity {} added.", act.format_name()),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        ),
                        Err(e) => self.bot_ref.tell(
                            SendMessageReply(
                                format!("Error creating activity. {:?}", e),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        ),
                    }
                }
                "addsc" => {
                    if args.len() < 2 {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "Syntax: /activities addsc ActivityID ShortcutName Game name"
                                    .into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }

                    let args: Vec<&str> = args[1].splitn(3, ' ').collect();
                    if args.len() != 3 {
                        return self.bot_ref.tell(SendMessageReply(
                            "To add a shortcut specify activity ID, shortcut name and then game name"
                                .into(),
                            message,
                            Format::Plain,
                            Notify::Off,
                        ), None);
                    }

                    let link = args[0].parse::<i32>();
                    if link.is_err() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "ActivityID must be a number".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }
                    let link = link.unwrap();
                    let name = args[1].to_string();
                    let game = args[2].to_string();

                    let act = Activity::find_one(&connection, &link).expect("Failed to run SQL");

                    if act.is_none() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                format!("Activity {} was not found.", link),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }

                    let shortcut = NewActivityShortcut { name, game, link };

                    if shortcut.save(&connection).is_err() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "Error creating shortcut".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }

                    self.bot_ref.tell(
                        SendMessageReply(
                            "Shortcut added".into(),
                            message,
                            Format::Plain,
                            Notify::Off,
                        ),
                        None,
                    );
                }
                "edit" => {
                    if args.len() < 2 {
                        self.bot_ref.tell(
                            SendMessageReply(
                                "Syntax: /activities edit ID KV".into(),
                                message.clone(),
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                        return self.usage(&message);
                    }

                    let args: Vec<&str> = args[1].splitn(2, ' ').collect();
                    if args.len() != 2 {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "To edit specify activity id and then KV pairs".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }

                    let id = args[0].parse::<i32>();
                    if id.is_err() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "ActivityID must be a number".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }
                    let id = id.unwrap();

                    let act = Activity::find_one(&connection, &id).expect("Failed to run SQL");

                    if act.is_none() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                format!("Activity {} was not found.", id),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }
                    let mut act = act.unwrap();

                    let argmap = parse_kv_args(args[1]);
                    if argmap.is_none() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "Invalid activity specification, see help.".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }
                    let argmap = argmap.unwrap();

                    for (key, val) in argmap {
                        match key {
                            "name" => act.name = val.into(),
                            "min_fireteam_size" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.bot_ref.tell(
                                        SendMessageReply(
                                            "min_fireteam_size must be a number".into(),
                                            message,
                                            Format::Plain,
                                            Notify::Off,
                                        ),
                                        None,
                                    );
                                }
                                act.min_fireteam_size = val.unwrap()
                            }
                            "max_fireteam_size" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.bot_ref.tell(
                                        SendMessageReply(
                                            "max_fireteam_size must be a number".into(),
                                            message,
                                            Format::Plain,
                                            Notify::Off,
                                        ),
                                        None,
                                    );
                                }
                                act.max_fireteam_size = val.unwrap()
                            }
                            "min_light" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.bot_ref.tell(
                                        SendMessageReply(
                                            "min_light must be a number".into(),
                                            message,
                                            Format::Plain,
                                            Notify::Off,
                                        ),
                                        None,
                                    );
                                }
                                act.min_light = Some(val.unwrap())
                            }
                            "min_level" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self.bot_ref.tell(
                                        SendMessageReply(
                                            "min_level must be a number".into(),
                                            message,
                                            Format::Plain,
                                            Notify::Off,
                                        ),
                                        None,
                                    );
                                }
                                act.min_level = Some(val.unwrap())
                            }
                            "mode" => act.mode = Some(val.into()),
                            _ => {
                                return self.bot_ref.tell(
                                    SendMessageReply(
                                        format!("Unknown field name {}", key),
                                        message,
                                        Format::Plain,
                                        Notify::Off,
                                    ),
                                    None,
                                );
                            }
                        }
                    }

                    match act.save(&connection) {
                        Ok(act) => self.bot_ref.tell(
                            SendMessageReply(
                                format!("Activity {} updated.", act.format_name()),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        ),
                        Err(e) => self.bot_ref.tell(
                            SendMessageReply(
                                format!("Error updating activity. {:?}", e),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        ),
                    }
                }
                "delete" => {
                    if args.len() < 2 {
                        self.bot_ref.tell(
                            SendMessageReply(
                                "Syntax: /activities delete ID".into(),
                                message.clone(),
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                        return self.usage(&message);
                    }

                    let id = args[1].parse::<i32>();
                    if id.is_err() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                "ActivityID must be a number".into(),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }
                    let id = id.unwrap();

                    let act = Activity::find_one(&connection, &id).expect("Failed to run SQL");

                    if act.is_none() {
                        return self.bot_ref.tell(
                            SendMessageReply(
                                format!("Activity {} was not found.", id),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        );
                    }

                    let act = act.unwrap();

                    let name = act.format_name();

                    match act.destroy(&connection) {
                        Ok(_) => self.bot_ref.tell(
                            SendMessageReply(
                                format!("Activity {} deleted.", name),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        ),
                        Err(e) => self.bot_ref.tell(
                            SendMessageReply(
                                format!("Error deleting activity. {:?}", e),
                                message,
                                Format::Plain,
                                Notify::Off,
                            ),
                            None,
                        ),
                    }
                }
                _ => {
                    self.bot_ref.tell(
                        SendMessageReply(
                            "Unknown activities operation".into(),
                            message.clone(),
                            Format::Plain,
                            Notify::Off,
                        ),
                        None,
                    );
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
