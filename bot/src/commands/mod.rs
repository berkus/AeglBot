use {
    crate::actors::bot_actor::{ActorUpdateMessage, Format, Notify, SendMessageReply},
    entity::guardians,
    kameo::actor::ActorRef,
    sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter},
};

#[macro_export]
macro_rules! command_actor {
    ($name:ident, $prefix:literal, $help:literal) => {
        use {
            kameo::{actor::ActorRef, error::Infallible, message::*, Actor},
            sea_orm::DatabaseConnection,
            $crate::BotCommand,
        };

        #[derive(Clone)]
        pub struct $name {
            bot_ref: ActorRef<$crate::actors::bot_actor::BotActor>,
            bot_name: String,
            connection_pool: DatabaseConnection,
        }

        impl BotCommand for $name {
            fn prefix() -> &'static str {
                concat!("/", $prefix)
            }

            fn description() -> &'static str {
                $help
            }
        }

        impl $name {
            pub fn new(
                bot_ref: ActorRef<$crate::actors::bot_actor::BotActor>,
                bot_name: String,
                connection_pool: DatabaseConnection,
            ) -> Self {
                Self {
                    bot_ref,
                    bot_name,
                    connection_pool,
                }
            }

            pub fn connection(&self) -> &DatabaseConnection {
                &self.connection_pool
            }

            pub async fn usage(&self, message: &$crate::actors::bot_actor::ActorUpdateMessage) {
                self.send_reply(
                    message,
                    $crate::render_template_or_err!(concat!($prefix, "/usage")),
                )
                .await;
            }

            #[allow(dead_code, reason = "help_command doesn't use those")]
            async fn send_reply_with_format<S>(
                &self,
                message: &$crate::actors::bot_actor::ActorUpdateMessage,
                reply: S,
                format: $crate::actors::bot_actor::Format,
            ) where
                S: Into<String>,
            {
                let _ = self
                    .bot_ref
                    .tell($crate::actors::bot_actor::SendMessageReply(
                        reply.into(),
                        message.clone(),
                        format,
                        $crate::actors::bot_actor::Notify::Off,
                    ))
                    .await;
            }

            #[allow(dead_code, reason = "help_command doesn't use those")]
            async fn send_reply<S>(
                &self,
                message: &$crate::actors::bot_actor::ActorUpdateMessage,
                reply: S,
            ) where
                S: Into<String>,
            {
                self.send_reply_with_format(
                    message,
                    reply,
                    $crate::actors::bot_actor::Format::Plain,
                )
                .await;
            }
        }

        impl Actor for $name {
            type Args = Self;
            type Error = Infallible;

            async fn on_start(
                args: Self::Args,
                _actor_ref: ActorRef<Self>,
            ) -> Result<Self, Self::Error> {
                Ok(args)
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
mod editguar_command;
pub use self::editguar_command::*;
mod help_command;
pub use self::help_command::*;
mod join_command;
pub use self::join_command::*;
mod lfg_command;
pub use self::lfg_command::*;
mod list_command;
pub use self::list_command::*;
mod manage_command;
pub use self::manage_command::*;
mod psn_command;
pub use self::psn_command::*;
mod uptime_command;
pub use self::uptime_command::*;
mod whois_command;
pub use self::whois_command::*;

pub fn decapitalize(s: &str) -> String {
    s.chars()
        .next()
        .map(|item| item.to_lowercase().chain(s.chars().skip(1)).collect())
        .unwrap()
}

/// Return a guardian record if message author is registered in Guardians table, `None` otherwise.
pub async fn validate_username(
    bot: &ActorRef<crate::actors::bot_actor::BotActor>,
    message: &ActorUpdateMessage,
    connection: &DatabaseConnection,
) -> Option<guardians::Model> {
    let username = match message.update.from.as_ref().unwrap().username {
        None => {
            let _ = bot
                .tell(SendMessageReply(
                    "❌ You have no telegram username, register your telegram account first."
                        .into(),
                    message.clone(),
                    Format::Plain,
                    Notify::Off,
                ))
                .await;
            return None;
        }
        Some(ref name) => name.clone(),
    };

    let db_user = guardians::Entity::find()
        .filter(guardians::Column::TelegramName.eq(&username)) // @todo Fix with tg-id
        .one(connection)
        .await;

    match db_user {
        Ok(Some(user)) => Some(user),
        Ok(None) => {
            let _ = bot
                .tell(SendMessageReply(
                    "❌ You need to link your PSN account first: use /psn command".into(),
                    message.clone(),
                    Format::Plain,
                    Notify::Off,
                ))
                .await;
            None
        }
        Err(e) => {
            let _ = bot
                .tell(SendMessageReply(
                    format!("❌ Error querying guardian info. {e}"),
                    message.clone(),
                    Format::Plain,
                    Notify::Off,
                ))
                .await;
            None
        }
    }
}

/// Return a guardian record if message author is an admin user, `None` otherwise.
pub async fn admin_check(
    bot: &ActorRef<crate::actors::bot_actor::BotActor>,
    message: &ActorUpdateMessage,
    connection: &DatabaseConnection,
) -> Option<guardians::Model> {
    validate_username(bot, message, connection)
        .await
        .filter(|u| u.is_admin)
}

pub async fn guardian_lookup(
    name: &str,
    connection: &DatabaseConnection,
) -> Result<Option<guardians::Model>, sea_orm::DbErr> {
    if let Some(name) = name.strip_prefix('@') {
        guardians::Entity::find()
            .filter(guardians::Column::TelegramName.eq(name))
            .one(connection)
            .await
    } else {
        guardians::Entity::find()
            .filter(guardians::Column::PsnName.contains(name))
            .one(connection)
            .await
    }
    // @todo: lookup by integer id, positive (tg user id)
}

/// Match command in both variations (with bot name and without bot name).
/// @param text Input message received from Telegram.
/// @param command Command name with leading slash, if it's a root command. FIXME: Is it correct?
/// @param bot_name Registered bot name.
/// @returns A pair of matched command and remainder of the message text.
/// (None, None) if command did not match,
/// (command, and Some remaining text after command otherwise).
pub fn match_command(
    text: Option<&str>,
    command: &str,
    bot_name: &str,
) -> (Option<String>, Option<String>) {
    // Take first token in the text - that must be the command, if any.
    // Split it by @ to see if we have a bot name attached
    // If we do - it must match our bot name completely.
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
                                .trim_start_matches('@')
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
    .unwrap_or((None, None))
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
