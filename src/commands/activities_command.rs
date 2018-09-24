use crate::models::{Activity, ActivityShortcut};
use crate::{Bot, BotCommand, DbConnection};
use diesel::{self, prelude::*};
use futures::Future;
use itertools::Itertools;
use std::collections::HashMap;

pub struct ActivitiesCommand;

command_ctor!(ActivitiesCommand);

impl ActivitiesCommand {
    fn usage(bot: &Bot, message: &telebot::objects::Message) {
        bot.send_plain_reply(&message, "".into());
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

        //        ☐ find activities ids
        //        ☐ `/activities ids` similar to `/activites` but for actual activities, not shortcuts.
        //        ☐ add new activities w/ shortcuts
        //        ☐ `/activities add key=value,key=value` e.g.
        //        ☐ `/activities add min_fireteam_size=1 max_fireteam_size=6 name="Last Wish, Enhance" mode="prestige"` etc. <- note "," in name - should parse
        //     allow shortcut=lastwn in these pairs as well -- will also allow editing shortcut (s?)
        //        ☐ `/activities edit ACTIVITY_ID key=value,key=value`
        //        ☐ `/activities addsc SHORTCUT ACTIVITY_ID`

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
                // min_fireteam_size=1 max_fireteam_size=6 name="Last Wish, Enhance" mode="prestige"
                // split by '='
                // ['min_fireteam_size', '1 max_fireteam_size', '6 name', '"Last Wish, Enhance" mode', '"prestige"']
                // .iter()
                // first = param name
                // next = if not last, then strip off next param name
                // if count > 2: take(1..n-1).rsplitn(2, ' ')

                // ['min_fireteam_size', '1 max_fireteam_size', '6 name', '"Last Wish, Enhance" mode', '"prestige"']
                //                let fragments = args.split('=');
                //
                //                if fragments.len() == 2 { // only single parameter
                //                } else {
                //                    // ['max_fireteam_size', '1', 'name', '6', 'mode', '"Last Wish, Enhance"']
                //                    let subfrags = fragments.take(1..fragments.len() - 1).iter().rsplitn(2, ' ').collect();
                //                }

                // parse key-value pairs, validate, check presence of mandatory
                // check no duplicates
                bot.send_plain_reply(&message, "ADD".into());
            }
            "addsc" => {
                bot.send_plain_reply(&message, "ADD SC".into());
            }
            "edit" => {
                bot.send_plain_reply(&message, "EDIT".into());
            }
            _ => {
                bot.send_plain_reply(&message, "Unknown activities operation".into());
            }
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
        let fragments: Vec<&str> = args.split('=').collect();

        println!("{:?}", fragments);

        if fragments.len() == 2 {
            // only single parameter
        } else {
            // ['max_fireteam_size', '1', 'name', '6', 'mode', '"Last Wish, Enhance"']
            let subfrags = itertools::Itertools::flatten(
                fragments[1..fragments.len() - 1].iter().map(|x: &&str| {
                    x.rsplitn(2, ' ')
                        .collect::<Vec<&str>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<&str>>()
                }),
            ).collect::<Vec<&str>>();

            println!("{:?}", subfrags);

            let mut final_ = vec![fragments[0]];
            final_.extend(subfrags);
            final_.extend(vec![fragments[fragments.len() - 1]]);

            println!("Final {:?}", final_);

            let the_map = final_
                .into_iter()
                .tuples()
                .map(|(k, v)| (k, v.trim_matches('"')))
                .collect::<HashMap<_, _>>();

            println!("As map {:?}", the_map);
        }
    }
}
