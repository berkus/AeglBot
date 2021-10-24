use {
    crate::{
        bot_actor::{ActorUpdateMessage, BotActorMsg, Format, Notify, SendMessageReply},
        DbConnection,
        {models::Guardian, schema::guardians::dsl::*},
    },
    diesel::prelude::*,
    riker::actors::{ActorRef, Tell},
};

#[macro_export]
macro_rules! command_actor {
    ($name:ident, [ $($msgs:ident),* ]) => {
        use crate::{bot_actor::BotActorMsg, NamedActor, DbConnPool, BotConnection};
        use riker::actors::{
            actor, Actor, ActorFactoryArgs, ActorRef, BasicActorRef, Context, Sender, Receive,
        };
        use paste::paste;

        #[derive(Clone)]
        #[actor($($msgs)*)]
        pub struct $name {
            bot_ref: ActorRef<BotActorMsg>,
            bot_name: String,
            connection_pool: DbConnPool,
        }

        impl $name {
            pub fn connection(&self) -> BotConnection {
                self.connection_pool.get().unwrap()
            }
        }

        impl NamedActor for $name {
            fn actor_name() -> String { std::stringify!($name).into() }
        }

        impl Actor for $name {
            type Msg = paste! { [<$name Msg>] };

            fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
                self.receive(ctx, msg, sender);
            }
        }

        impl ActorFactoryArgs<(ActorRef<BotActorMsg>, String, DbConnPool)> for $name {
            fn create_args((bot_ref, bot_name, connection_pool): (ActorRef<BotActorMsg>, String, DbConnPool)) -> Self {
                Self { bot_ref, bot_name, connection_pool }
            }
        }
    };
}

mod activities_command;
pub use self::activities_command::*;
mod cancel_command;
pub use self::cancel_command::*;
mod chatid_command;
pub use self::chatid_command::*;
mod d2week_command;
pub use self::d2week_command::*;
mod dweek_command;
pub use self::dweek_command::*;
mod edit_command;
pub use self::edit_command::*;
// mod editguar_command;
// pub use self::editguar_command::*;
mod help_command;
pub use self::help_command::*;
mod info_command;
pub use self::info_command::*;
mod join_command;
pub use self::join_command::*;
mod lfg_command;
pub use self::lfg_command::*;
mod list_command;
pub use self::list_command::*;
// mod manage_command;
// pub use self::manage_command::*;
mod psn_command;
pub use self::psn_command::*;
mod whois_command;
pub use self::whois_command::*;

pub fn decapitalize(s: &str) -> String {
    s.chars()
        .nth(0)
        .map(|item| item.to_lowercase().chain(s.chars().skip(1)).collect())
        .unwrap()
}

/// Return a guardian record if message author is registered in Guardians table, `None` otherwise.
pub fn validate_username(
    bot: &ActorRef<BotActorMsg>,
    message: &ActorUpdateMessage,
    connection: &DbConnection,
) -> Option<Guardian> {
    let username = match message.update.from().as_ref().unwrap().username {
        None => {
            bot.tell(
                SendMessageReply(
                    "You have no telegram username, register your telegram account first.".into(),
                    message.clone(),
                    Format::Plain,
                    Notify::Off,
                ),
                None,
            );
            return None;
        }
        Some(ref name) => name.clone(),
    };

    let db_user = guardians
        .filter(telegram_name.eq(&username)) // @todo Fix with tg-id
        .first::<Guardian>(connection)
        .optional();

    match db_user {
        Ok(Some(user)) => Some(user),
        Ok(None) => {
            bot.tell(
                SendMessageReply(
                    "You need to link your PSN account first: use /psn command".into(),
                    message.clone(),
                    Format::Plain,
                    Notify::Off,
                ),
                None,
            );
            None
        }
        Err(_) => {
            bot.tell(
                SendMessageReply(
                    "Error querying guardian info.".into(),
                    message.clone(),
                    Format::Plain,
                    Notify::Off,
                ),
                None,
            );
            None
        }
    }
}

/// Return a guardian record if message author is an admin user, `None` otherwise.
pub fn admin_check(
    bot: &ActorRef<BotActorMsg>,
    message: &ActorUpdateMessage,
    connection: &DbConnection,
) -> Option<Guardian> {
    validate_username(bot, message, connection).filter(|g| g.is_admin)
}

pub fn guardian_lookup(
    name: &str,
    connection: &DbConnection,
) -> Result<Option<Guardian>, diesel::result::Error> {
    if name.starts_with('@') {
        guardians
            .filter(telegram_name.eq(&name[1..]))
            .first::<Guardian>(connection)
            .optional()
    } else {
        guardians
            .filter(psn_name.ilike(&name))
            .first::<Guardian>(connection)
            .optional()
    }
    // @todo: lookup by integer id, positive
}

/// Match command in both variations (with bot name and without bot name).
/// @param msg Input message received from Telegram.
/// @param command Command name with leading slash, if it's a root command. FIXME: Is it correct?
/// @param bot_name Registered bot name.
/// @returns A pair of matched command and remainder of the message text.
/// (None, None) if command did not match,
/// (command, and Some remaining text after command otherwise).
fn match_command(
    text: Option<&str>,
    command: &str,
    bot_name: &str,
) -> (Option<String>, Option<String>) {
    // Take first token in the text - that must be the command, if any.
    // Split it by @ to see if we have a bot name attached
    // If we do - it must match out bot name completely.
    // Strip trailing numeric digits from the left side - this might be part of the command argument, remember it.
    // The rest of the left side must match EXACTLY, not as a prefix.
    text.and_then(|input| {
        log::debug!("matching {:#?} against {:#?}", input, command);
        input
            .split_whitespace()
            .next()
            .and_then(|s| {
                let mut matches = s.split('@');
                let cmd = matches.next();
                if let Some(bot) = matches.next() {
                    if bot != bot_name {
                        log::debug!(".. some other bot matched");
                        return None;
                    }
                }
                cmd
            })
            .and_then(|cmd| {
                // Some clients send /cancel593@AeglBot on click, so strip the numeric code at the end, if any.
                let cmd = cmd.trim_end_matches(char::is_numeric);
                if cmd != command {
                    return None;
                }
                log::debug!(".. matched");
                Some((
                    Some(cmd.into()),
                    input
                        .get(command.len()..)
                        .map(|x| {
                            let prefix = x
                                .chars()
                                .take_while(|c| c.is_numeric())
                                .fold(String::new(), |a, b| format!("{}{}", a, b));
                            log::debug!("Matched prefix {}", prefix);
                            // "593@AeglBot something something"
                            let x = x
                                .trim_start_matches(&prefix)
                                .trim_start_matches("@")
                                .trim_start_matches(bot_name)
                                .trim_start();
                            // TODO: Prepend the ID stripped from the command previously
                            // "593 something something"
                            format!("{} {}", prefix, x).trim().to_string()
                        })
                        .filter(|y| !y.is_empty()),
                ))
            })
    })
    .or(Some((None, None)))
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::match_command;

    #[test]
    fn test_command_match_without_bot_name_and_no_prefix() {
        assert_eq!(
            match_command(Some("/cmd some text"), "/cmd", "BotName"),
            (Some("/cmd".into()), Some("some text".into()))
        );
    }

    #[test]
    fn test_command_match_without_bot_name_and_no_prefix_and_one_arg() {
        assert_eq!(
            match_command(Some("/cmd66"), "/cmd", "BotName"),
            (Some("/cmd".into()), Some("66".into()))
        );
    }

    #[test]
    fn test_command_did_not_match_without_bot_name_and_no_prefix() {
        assert_eq!(
            match_command(Some("/othercmd some text"), "/cmd", "BotName"),
            (None, None)
        );
    }

    // ..all the way to

    #[test]
    fn test_command_match_with_bot_name_and_prefix() {
        assert_eq!(
            match_command(Some("/cmd735@TestBot some text"), "/cmd", "TestBot"),
            (Some("/cmd".into()), Some("735 some text".into()))
        );
    }

    #[test]
    fn test_command_did_not_match_with_bot_name_and_prefix() {
        assert_eq!(
            match_command(Some("/othercmd735@TestBot some text"), "/cmd", "TestBot"),
            (None, None)
        );
    }
}
