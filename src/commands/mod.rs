use {
    crate::{
        bot_actor::{ActorUpdateMessage, BotActor, BotActorMsg, Format, Notify, SendMessageReply},
        DbConnection,
        {models::Guardian, schema::guardians::dsl::*},
    },
    diesel::prelude::*,
    futures::Future,
    riker::actors::{ActorRef, Tell},
    teloxide::prelude::*,
};

#[macro_export]
macro_rules! command_actor {
    ($name:ident, [ $($msgs:ident),* ]) => {
        use crate::{bot_actor::BotActorMsg, NamedActor};
        use riker::actors::{
            actor, Actor, ActorFactoryArgs, ActorRef, BasicActorRef, Context, Sender,
        };
        use paste::paste;

        #[derive(Clone)]
        #[actor($($msgs)*)]
        pub struct $name {
            bot_ref: ActorRef<BotActorMsg>,
        }

        impl NamedActor for $name {
            fn name() -> String { std::stringify!($name).into() }
        }

        impl Actor for $name {
            type Msg = paste! { [<$name Msg>] };

            fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
                self.receive(ctx, msg, sender);
            }
        }

        impl ActorFactoryArgs<ActorRef<BotActorMsg>> for $name {
            fn create_args(bot_ref: ActorRef<BotActorMsg>) -> Self {
                Self { bot_ref }
            }
        }
    };
}

// mod activities_command;
// pub use self::activities_command::*;
// mod cancel_command;
// pub use self::cancel_command::*;
// mod chatid_command;
// pub use self::chatid_command::*;
// mod d2week_command;
// pub use self::d2week_command::*;
// mod dweek_command;
// pub use self::dweek_command::*;
// mod edit_command;
// pub use self::edit_command::*;
// mod editguar_command;
// pub use self::editguar_command::*;
// mod help_command;
// pub use self::help_command::*;
mod info_command;
pub use self::info_command::*;
// mod join_command;
// pub use self::join_command::*;
// mod lfg_command;
// pub use self::lfg_command::*;
// mod list_command;
// pub use self::list_command::*;
// mod manage_command;
// pub use self::manage_command::*;
// mod psn_command;
// pub use self::psn_command::*;
// mod whois_command;
// pub use self::whois_command::*;

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
/// @param data Input text received from Telegram.
/// @param command Command name without leading slash.
/// @param bot_name Registered bot name.
/// @returns A pair of matched command and remainder of the message text.
/// (None, None) if command did not match,
/// (command, and Some remaining text after command otherwise).
fn match_command(
    msg: &ActorUpdateMessage,
    command: &str,
    bot_name: &str,
) -> (Option<String>, Option<String>) {
    // Take first token in the text - that must be the command, if any.
    // Split it by @ to see if we have a bot name attached
    // If we do - it must match out bot name completely.
    // Strip trailing numeric digits from the left side - this might be part of the command argument, remember it.
    // The rest of the left side must match EXACTLY, not as a prefix.
    msg.update
        .text()
        .map(|data| {
            log::debug!("matching {:#?} against {:#?}", data, command);
            data.split_whitespace()
                .next()
                .map(|s| {
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
                .unwrap_or(None)
                .map(|cmd| {
                    // Some clients send /cancel593@AeglBot on click, so strip the numeric code at the end, if any.
                    let cmd = cmd.trim_end_matches(|c: char| c.is_digit(10));
                    if cmd != command {
                        return None;
                    }
                    log::debug!(".. matched");
                    Some((
                        Some(cmd.into()),
                        data.get(command.len()..)
                            .map(|x| x.trim_start().to_string())
                            .filter(|y| !y.is_empty()),
                    ))
                })
                .unwrap_or(None)
        })
        .unwrap_or(None)
        .or(Some((None, None)))
        .unwrap()
}
