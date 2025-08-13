use {
    crate::{
        bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
        commands::{admin_check, match_command},
        BotCommand,
    },
    entity::{activities, activityshortcuts},
    itertools::Itertools,
    kameo::message::Context,
    sea_orm::{
        ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    },
    std::collections::HashMap,
};

command_actor!(ActivitiesCommand, [ActorUpdateMessage]);

impl ActivitiesCommand {
    async fn activities_usage(&self, message: &ActorUpdateMessage) {
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
        )
        .await;
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

impl Message<ActorUpdateMessage> for ActivitiesCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handle_message(message).await;
    }
}

impl ActivitiesCommand {
    async fn handle_message(&self, message: ActorUpdateMessage) {
        let connection = self.connection();
        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            if args.is_none() {
                // Just /activities
                let games = activityshortcuts::Entity::find()
                    .select_only()
                    .column(activityshortcuts::Column::Game)
                    .distinct()
                    .order_by_asc(activityshortcuts::Column::Game)
                    .all(connection)
                    .await
                    .expect("Failed to load activity shortcuts");

                let mut text = "Activities: use a short name:\n".to_owned();

                for game_row in games {
                    let game_name = game_row.game;
                    text += &format!("*** <b>{0}</b>:\n", game_name);
                    let shortcuts = activityshortcuts::Entity::find()
                        .filter(activityshortcuts::Column::Game.eq(&game_name))
                        .order_by_asc(activityshortcuts::Column::Name)
                        .all(connection)
                        .await
                        .expect("TEMP loading @FIXME");

                    for shortcut in shortcuts {
                        let link_activity = activities::Entity::find_by_id(shortcut.link)
                            .one(connection)
                            .await
                            .expect("Failed to load activity")
                            .expect("Activity not found");

                        text += &format!(
                            "<b>{name}</b>\t{link}\n",
                            name = shortcut.name,
                            link = link_activity.format_name(),
                        );
                    }
                    text += "\n";
                }

                let _ = self
                    .bot_ref
                    .tell(SendMessageReply(text, message, Format::Html, Notify::Off))
                    .await;
                return;
            }

            // some args - pass to a subcommand
            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(2, ' ').collect();

            if args.is_empty() {
                self.activities_usage(&message).await;
                return;
            }

            let admin = admin_check(&self.bot_ref, &message, connection).await;
            if admin.is_none() {
                self.send_reply(&message, "You are not admin").await;
                return;
            }

            // split into subcommands:
            match args[0] {
                "ids" => {
                    let games = activities::Entity::find()
                        .all(connection)
                        .await
                        .expect("Failed to load activities");

                    let mut text = "Activities:\n\n".to_string();
                    for activity in games {
                        text += &format!(
                            "{}. {} {}\n",
                            activity.id,
                            activity.name,
                            activity.mode.unwrap_or("".into())
                        );
                    }
                    self.send_reply(&message, text).await;
                }
                "add" => {
                    if args.len() < 2 {
                        self.send_reply(&message, "Syntax: /activities add KV")
                            .await;
                        return self.activities_usage(&message).await;
                    }

                    let argmap = parse_kv_args(args[1]);
                    if argmap.is_none() {
                        return self
                            .send_reply(&message, "Invalid activity specification, see help.")
                            .await;
                    }
                    let mut argmap = argmap.unwrap();
                    let name = argmap.remove("name");
                    if name.is_none() {
                        return self
                            .send_reply(&message, "Must specify activity name, see help.")
                            .await;
                    }

                    let min_fireteam_size = argmap.remove("min_fireteam_size");
                    if min_fireteam_size.is_none() {
                        return self
                            .send_reply(&message, "Must specify min_fireteam_size, see help.")
                            .await;
                    }
                    let min_fireteam_size = min_fireteam_size.unwrap().parse::<i32>();
                    if min_fireteam_size.is_err() {
                        return self
                            .send_reply(&message, "min_fireteam_size must be a number")
                            .await;
                    }
                    let min_fireteam_size = min_fireteam_size.unwrap();

                    let max_fireteam_size = argmap.remove("max_fireteam_size");
                    if max_fireteam_size.is_none() {
                        return self
                            .send_reply(&message, "Must specify max_fireteam_size, see help.")
                            .await;
                    }
                    let max_fireteam_size = max_fireteam_size.unwrap().parse::<i32>();
                    if max_fireteam_size.is_err() {
                        return self
                            .send_reply(&message, "max_fireteam_size must be a number")
                            .await;
                    }
                    let max_fireteam_size = max_fireteam_size.unwrap();

                    // check no duplicates -- ?
                    let mut act = activities::ActiveModel {
                        name: Set(name.unwrap().to_string()),
                        mode: Set(None),
                        min_fireteam_size: Set(min_fireteam_size),
                        max_fireteam_size: Set(max_fireteam_size),
                        min_level: Set(None),
                        min_light: Set(None),
                        ..Default::default()
                    };

                    for (key, val) in argmap {
                        match key {
                            "min_light" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self
                                        .send_reply(&message, "min_light must be a number")
                                        .await;
                                }
                                act.min_light = Set(Some(val.unwrap()));
                            }
                            "min_level" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self
                                        .send_reply(&message, "min_level must be a number")
                                        .await;
                                }
                                act.min_level = Set(Some(val.unwrap()));
                            }
                            "mode" => act.mode = Set(Some(val.to_string())),
                            _ => {
                                return self
                                    .send_reply(&message, format!("Unknown field name {}", key))
                                    .await;
                            }
                        }
                    }

                    match act.insert(connection).await {
                        Ok(act) => {
                            self.send_reply(
                                &message,
                                format!("Activity {} added.", act.format_name()),
                            )
                            .await
                        }
                        Err(e) => {
                            self.send_reply(&message, format!("Error creating activity. {:?}", e))
                                .await
                        }
                    }
                }
                "addsc" => {
                    if args.len() < 2 {
                        return self
                            .send_reply(
                                &message,
                                "Syntax: /activities addsc ActivityID ShortcutName Game name",
                            )
                            .await;
                    }

                    let args: Vec<&str> = args[1].splitn(3, ' ').collect();
                    if args.len() != 3 {
                        return self.send_reply(
                            &message,
                            "To add a shortcut specify activity ID, shortcut name and then the game name",
                        ).await;
                    }

                    let link = args[0].parse::<i32>();
                    if link.is_err() {
                        return self
                            .send_reply(&message, "ActivityID must be a number")
                            .await;
                    }
                    let link = link.unwrap();
                    let name = args[1].to_string();
                    let game = args[2].to_string();

                    let act = activities::Entity::find_by_id(link)
                        .one(connection)
                        .await
                        .expect("Failed to run SQL");

                    if act.is_none() {
                        return self
                            .send_reply(&message, format!("Activity {} was not found.", link))
                            .await;
                    }

                    let shortcut = activityshortcuts::ActiveModel {
                        name: Set(name),
                        game: Set(game),
                        link: Set(link),
                        ..Default::default()
                    };

                    if shortcut.insert(connection).await.is_err() {
                        return self.send_reply(&message, "Error creating shortcut").await;
                    }

                    self.send_reply(&message, "Shortcut added").await;
                }
                "edit" => {
                    if args.len() < 2 {
                        self.send_reply(&message, "Syntax: /activities edit ID KV")
                            .await;
                        return self.activities_usage(&message).await;
                    }

                    let args: Vec<&str> = args[1].splitn(2, ' ').collect();
                    if args.len() != 2 {
                        return self
                            .send_reply(
                                &message,
                                "To edit first specify Activity ID and then key=value pairs",
                            )
                            .await;
                    }

                    let id = args[0].parse::<i32>();
                    if id.is_err() {
                        return self
                            .send_reply(&message, "ActivityID must be a number")
                            .await;
                    }
                    let id = id.unwrap();

                    let act = activities::Entity::find_by_id(id)
                        .one(connection)
                        .await
                        .expect("Failed to run SQL");

                    if act.is_none() {
                        return self
                            .send_reply(&message, format!("Activity {} was not found.", id))
                            .await;
                    }
                    let act = act.unwrap();
                    let mut act: activities::ActiveModel = act.into();

                    let argmap = parse_kv_args(args[1]);
                    if argmap.is_none() {
                        return self
                            .send_reply(&message, "Invalid activity specification, see help.")
                            .await;
                    }
                    let argmap = argmap.unwrap();

                    for (key, val) in argmap {
                        match key {
                            "name" => act.name = Set(val.to_string()),
                            "min_fireteam_size" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self
                                        .send_reply(&message, "min_fireteam_size must be a number")
                                        .await;
                                }
                                act.min_fireteam_size = Set(val.unwrap())
                            }
                            "max_fireteam_size" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self
                                        .send_reply(&message, "max_fireteam_size must be a number")
                                        .await;
                                }
                                act.max_fireteam_size = Set(val.unwrap())
                            }
                            "min_light" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self
                                        .send_reply(&message, "min_light must be a number")
                                        .await;
                                }
                                act.min_light = Set(Some(val.unwrap()))
                            }
                            "min_level" => {
                                let val = val.parse::<i32>();
                                if val.is_err() {
                                    return self
                                        .send_reply(&message, "min_level must be a number")
                                        .await;
                                }
                                act.min_level = Set(Some(val.unwrap()))
                            }
                            "mode" => act.mode = Set(Some(val.to_string())),
                            _ => {
                                return self
                                    .send_reply(&message, format!("Unknown field name {}", key))
                                    .await;
                            }
                        }
                    }

                    match act.update(connection).await {
                        Ok(act) => {
                            self.send_reply(
                                &message,
                                format!("Activity {} updated.", act.format_name()),
                            )
                            .await
                        }
                        Err(e) => {
                            self.send_reply(&message, format!("Error updating activity. {:?}", e))
                                .await
                        }
                    }
                }
                "delete" => {
                    if args.len() < 2 {
                        self.send_reply(&message, "Syntax: /activities delete ID")
                            .await;
                        return self.activities_usage(&message).await;
                    }

                    let id = args[1].parse::<i32>();
                    if id.is_err() {
                        return self
                            .send_reply(&message, "ActivityID must be a number")
                            .await;
                    }
                    let id = id.unwrap();

                    let act = activities::Entity::find_by_id(id)
                        .one(connection)
                        .await
                        .expect("Failed to run SQL");

                    if act.is_none() {
                        return self
                            .send_reply(&message, format!("Activity {} was not found.", id))
                            .await;
                    }
                    let act = act.unwrap();

                    let name = act.format_name();

                    match activities::Entity::delete_by_id(id).exec(connection).await {
                        Ok(_) => {
                            self.send_reply(&message, format!("Activity {} deleted.", name))
                                .await
                        }
                        Err(e) => {
                            self.send_reply(&message, format!("Error deleting activity. {:?}", e))
                                .await
                        }
                    }
                }
                _ => {
                    self.send_reply(&message, "Unknown activities operation")
                        .await;
                    self.activities_usage(&message).await;
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
        2 =>
        // only single parameter
        {
            Some(final_collect(fragments))
        }
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
