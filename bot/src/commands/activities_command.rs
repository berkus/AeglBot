use {
    crate::{
        actors::bot_actor::{ActorUpdateMessage, Format},
        commands::{admin_check, match_command},
        render_template_or_err,
    },
    entity::{activities, activityshortcuts},
    itertools::Itertools,
    kameo::message::Context,
    sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set},
    std::collections::HashMap,
};

command_actor!(
    ActivitiesCommand,
    "activities",
    "List available activity shortcuts"
);

impl ActivitiesCommand {
    async fn all_activities_list(
        &self,
        connection: &DatabaseConnection,
        message: &ActorUpdateMessage,
    ) {
        let games = activityshortcuts::Entity::find()
            .find_also_related(activities::Entity)
            .order_by_asc(activityshortcuts::Column::Game)
            .order_by_asc(activityshortcuts::Column::Name)
            .all(connection)
            .await
            .expect("❌ Failed to load activity shortcuts");

        #[derive(serde::Serialize)]
        struct Game {
            game: String,
            shortcut: String,
            activity: String,
        }

        let games: Vec<Game> = games
            .into_iter()
            .map(|game| {
                let link_activity = game.1.expect("❌ Activity not found");

                Game {
                    game: game.0.game,
                    shortcut: game.0.name,
                    activity: link_activity.format_name(),
                }
            })
            .collect();

        self.send_reply_with_format(
            message,
            render_template_or_err!("activities/list", ("games" => &games)),
            Format::Html,
        )
        .await;
    }

    async fn activities_ids_list(
        &self,
        connection: &DatabaseConnection,
        message: &ActorUpdateMessage,
    ) {
        let games = activities::Entity::find()
            .all(connection)
            .await
            .expect("❌ Failed to load activities");

        let mut text = "Activities:\n\n".to_string();
        for activity in games {
            text += &format!(
                "{}. {} {}\n",
                activity.id,
                activity.name,
                activity.mode.unwrap_or("".into())
            );
        }
        self.send_reply(message, text).await;
    }

    async fn activity_add(
        &self,
        connection: &DatabaseConnection,
        message: &ActorUpdateMessage,
        mut argmap: HashMap<&str, &str>,
    ) {
        let name = argmap.remove("name");
        if name.is_none() {
            return self
                .send_reply(message, "❌ Must specify activity name, see help.")
                .await;
        }

        let min_fireteam_size = argmap.remove("min_fireteam_size");
        if min_fireteam_size.is_none() {
            return self
                .send_reply(message, "❌ Must specify min_fireteam_size, see help.")
                .await;
        }
        let min_fireteam_size = min_fireteam_size.unwrap().parse::<i32>();
        if min_fireteam_size.is_err() {
            return self
                .send_reply(message, "❌ min_fireteam_size must be a number")
                .await;
        }
        let min_fireteam_size = min_fireteam_size.unwrap();

        let max_fireteam_size = argmap.remove("max_fireteam_size");
        if max_fireteam_size.is_none() {
            return self
                .send_reply(message, "❌ Must specify max_fireteam_size, see help.")
                .await;
        }
        let max_fireteam_size = max_fireteam_size.unwrap().parse::<i32>();
        if max_fireteam_size.is_err() {
            return self
                .send_reply(message, "❌ max_fireteam_size must be a number")
                .await;
        }
        let max_fireteam_size = max_fireteam_size.unwrap();

        // TODO: check for no duplicates -- ?

        let mut act = activities::ActiveModel {
            name: Set(name.unwrap().to_string()),
            mode: Set(None),
            min_fireteam_size: Set(min_fireteam_size),
            max_fireteam_size: Set(max_fireteam_size),
            min_level: Set(None),
            min_light: Set(None),
            ..Default::default()
        };

        let min_light = argmap.remove("min_light");
        if let Some(min_light) = min_light {
            let val = min_light.parse::<i32>();
            if val.is_err() {
                return self
                    .send_reply(message, "❌ min_light must be a number")
                    .await;
            }
            act.min_light = Set(Some(val.unwrap()));
        }

        let min_level = argmap.remove("min_level");
        if let Some(min_level) = min_level {
            let val = min_level.parse::<i32>();
            if val.is_err() {
                return self
                    .send_reply(message, "❌ min_level must be a number")
                    .await;
            }
            act.min_level = Set(Some(val.unwrap()));
        }

        let mode = argmap.remove("mode");
        if let Some(mode) = mode {
            act.mode = Set(Some(mode.to_string()));
        }

        let link = match act.insert(connection).await {
            Ok(act) => {
                self.send_reply(message, format!("✅ Activity {} added.", act.format_name()))
                    .await;
                act.id
            }
            Err(e) => {
                return self
                    .send_reply(message, format!("❌ Error creating activity. {:?}", e))
                    .await;
            }
        };

        if argmap.contains_key("shortcut") && argmap.contains_key("game") {
            let game = argmap.remove("game").unwrap().to_string();
            let shortcut = argmap.remove("shortcut").unwrap().to_string();
            self.activity_add_shortcut(connection, message, link, shortcut, game)
                .await;
        }
    }

    async fn activity_add_shortcut(
        &self,
        connection: &DatabaseConnection,
        message: &ActorUpdateMessage,
        link: i32,
        name: String,
        game: String,
    ) {
        let act = activities::Entity::find_by_id(link)
            .one(connection)
            .await
            .expect("Failed to run SQL");

        if act.is_none() {
            return self
                .send_reply(message, format!("❌ Activity {} was not found.", link))
                .await;
        }

        let shortcut = activityshortcuts::ActiveModel {
            name: Set(name),
            game: Set(game),
            link: Set(link),
            ..Default::default()
        };

        if shortcut.insert(connection).await.is_err() {
            return self.send_reply(message, "❌ Error creating shortcut").await;
        }

        self.send_reply(message, "✅ Shortcut added").await;
    }

    async fn activity_edit(
        &self,
        connection: &DatabaseConnection,
        message: &ActorUpdateMessage,
        id: i32,
        argmap: HashMap<&str, &str>,
    ) {
        let act = activities::Entity::find_by_id(id)
            .one(connection)
            .await
            .expect("❌ Failed to run SQL");

        if act.is_none() {
            return self
                .send_reply(message, format!("❌ Activity {} was not found.", id))
                .await;
        }
        let act = act.unwrap();
        let mut act: activities::ActiveModel = act.into();

        for (key, val) in argmap {
            match key {
                "name" => act.name = Set(val.to_string()),
                "min_fireteam_size" => {
                    let val = val.parse::<i32>();
                    if val.is_err() {
                        return self
                            .send_reply(message, "❌ min_fireteam_size must be a number")
                            .await;
                    }
                    act.min_fireteam_size = Set(val.unwrap())
                }
                "max_fireteam_size" => {
                    let val = val.parse::<i32>();
                    if val.is_err() {
                        return self
                            .send_reply(message, "❌ max_fireteam_size must be a number")
                            .await;
                    }
                    act.max_fireteam_size = Set(val.unwrap())
                }
                "min_light" => {
                    let val = val.parse::<i32>();
                    if val.is_err() {
                        return self
                            .send_reply(message, "❌ min_light must be a number")
                            .await;
                    }
                    act.min_light = Set(Some(val.unwrap()))
                }
                "min_level" => {
                    let val = val.parse::<i32>();
                    if val.is_err() {
                        return self
                            .send_reply(message, "❌ min_level must be a number")
                            .await;
                    }
                    act.min_level = Set(Some(val.unwrap()))
                }
                "mode" => act.mode = Set(Some(val.to_string())),
                _ => {
                    return self
                        .send_reply(message, format!("❌ Unknown field name {}", key))
                        .await;
                }
            }
        }

        match act.update(connection).await {
            Ok(act) => {
                self.send_reply(
                    message,
                    format!("✅ Activity {} updated.", act.format_name()),
                )
                .await
            }
            Err(e) => {
                self.send_reply(message, format!("❌ Error updating activity. {:?}", e))
                    .await
            }
        }
    }

    async fn activity_delete(
        &self,
        connection: &DatabaseConnection,
        message: &ActorUpdateMessage,
        id: i32,
    ) {
        let act = activities::Entity::find_by_id(id)
            .one(connection)
            .await
            .expect("❌ Failed to run SQL");

        if act.is_none() {
            return self
                .send_reply(message, format!("❌ Activity {} was not found.", id))
                .await;
        }
        let act = act.unwrap();

        let name = act.format_name();

        match activities::Entity::delete_by_id(id).exec(connection).await {
            Ok(_) => {
                self.send_reply(message, format!("✅ Activity {} deleted.", name))
                    .await
            }
            Err(e) => {
                // TODO: error chain?
                self.send_reply(message, format!("❌ Error deleting activity. {:?}", e))
                    .await
            }
        }
    }
}

impl Message<ActorUpdateMessage> for ActivitiesCommand {
    type Reply = ();

    async fn handle(
        &mut self,
        message: ActorUpdateMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let (Some(_), args) =
            match_command(message.update.text(), Self::prefix(), &self.bot_name)
        {
            let connection = self.connection();

            if args.is_none() {
                // Just /activities
                return self.all_activities_list(connection, &message).await;
            }

            // some args - pass to a subcommand
            let args = args.unwrap();
            let args: Vec<&str> = args.splitn(2, ' ').collect();

            if args.is_empty() {
                return self.usage(&message).await;
            }

            let admin = admin_check(&self.bot_ref, &message, connection).await;
            if admin.is_none() {
                return self.send_reply(&message, "❌ You are not admin").await;
            }

            // split into subcommands:
            match args[0] {
                "ids" => self.activities_ids_list(connection, &message).await,
                "add" => {
                    if args.len() < 2 {
                        self.send_reply(&message, "❓ Syntax: /activities add KV")
                            .await;
                        return self.usage(&message).await;
                    }

                    let argmap = parse_kv_args(args[1]);
                    if argmap.is_none() {
                        return self
                            .send_reply(&message, "❌ Invalid activity specification, see help.")
                            .await;
                    }
                    let argmap = argmap.unwrap();
                    self.activity_add(connection, &message, argmap).await;
                }
                "addsc" => {
                    if args.len() < 2 {
                        return self
                            .send_reply(
                                &message,
                                "❓ Syntax: /activities addsc ActivityID ShortcutName Game name",
                            )
                            .await;
                    }

                    let args: Vec<&str> = args[1].splitn(3, ' ').collect();
                    if args.len() != 3 {
                        return self.send_reply(
                            &message,
                            "❌ To add a shortcut specify 1) activity ID, 2) shortcut name and then 3) the game name",
                        ).await;
                    }

                    let link = args[0].parse::<i32>();
                    if link.is_err() {
                        return self
                            .send_reply(&message, "❌ ActivityID must be a number")
                            .await;
                    }
                    let link = link.unwrap();
                    let name = args[1].to_string();
                    let game = args[2].to_string();

                    self.activity_add_shortcut(connection, &message, link, name, game)
                        .await;
                }
                "edit" => {
                    if args.len() < 2 {
                        self.send_reply(&message, "❓ Syntax: /activities edit ID KV")
                            .await;
                        return self.usage(&message).await;
                    }

                    let args: Vec<&str> = args[1].splitn(2, ' ').collect();
                    if args.len() != 2 {
                        return self
                            .send_reply(
                                &message,
                                "❌ To edit first specify Activity ID and then key=value pairs",
                            )
                            .await;
                    }

                    let id = args[0].parse::<i32>();
                    if id.is_err() {
                        return self
                            .send_reply(&message, "❌ ActivityID must be a number")
                            .await;
                    }
                    let id = id.unwrap();

                    let argmap = parse_kv_args(args[1]);
                    if argmap.is_none() {
                        return self
                            .send_reply(&message, "❌ Invalid activity specification, see help.")
                            .await;
                    }
                    let argmap = argmap.unwrap();

                    self.activity_edit(connection, &message, id, argmap).await;
                }
                "delete" => {
                    if args.len() < 2 {
                        self.send_reply(&message, "❓ Syntax: /activities delete ID")
                            .await;
                        return self.usage(&message).await;
                    }

                    let id = args[1].parse::<i32>();
                    if id.is_err() {
                        return self
                            .send_reply(&message, "❌ Activity ID must be a number")
                            .await;
                    }
                    let id = id.unwrap();

                    self.activity_delete(connection, &message, id).await;
                }
                _ => {
                    self.send_reply(&message, "❌ Unknown activities operation")
                        .await;
                    self.usage(&message).await;
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
