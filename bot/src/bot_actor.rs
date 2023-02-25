use {
    crate::{
        commands::*, services::reminder_actor::ReminderActor, BotCommand, DbConnPool, NamedActor,
    },
    dotenv::dotenv,
    ractor::{cast, Actor, ActorProcessingErr, ActorRef},
    std::{env, fmt::Formatter},
    teloxide::{
        prelude::*,
        types::{ChatId, ParseMode},
    },
};

#[derive(Clone, Debug)]
pub enum Format {
    Plain,
    Markdown,
    Html,
}

#[derive(Clone, Debug)]
pub enum Notify {
    Off,
    On,
}

pub enum BotActorMsg {
    SendMessage(String, ChatId, Format, Notify),
    SendMessageReply(String, CommandMsg, Format, Notify),
    ListCommands(CommandMsg),
    RawCommand(CommandMsg), // Delivered to registered commands.
}

// #[derive(Clone)]
// #[actor(SendMessage, SendMessageReply, ListCommands)]
pub struct BotActor {
    pub bot: Bot,
    bot_name: String,
    lfg_chat_id: i64,
    // update_channel: ChannelRef<ActorUpdateMessage>,
    connection_pool: DbConnPool,
}

// impl std::fmt::Debug for BotActor {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "BotActor")
//     }
// }

/// A message delivered to command actor from telegram bot.
pub struct CommandMsg(Message);

impl BotActor {
    // Public API

    pub fn new(
        name: &str,
        bot: Bot,
        // chan: ChannelRef<ActorUpdateMessage>,
        lfg_chat_id: i64,
    ) -> Self {
        BotActor {
            bot,
            bot_name: name.to_string(),
            lfg_chat_id,
            // update_channel: chan,
            connection_pool: Self::establish_connection(),
        }
    }

    pub fn list_commands(&self, state: &mut <Self as Actor>::State) -> Vec<(String, String)> {
        state.commands_list.clone()
    }

    // Internal helpers

    // fn handle_error(error: anyhow::Error) -> RetryPolicy<anyhow::Error> {
    //     // count errors
    //     log::error!("handle_error");
    //     match error.downcast_ref::<anyhow::Error>() {
    //         Some(te) => {
    //             log::error!("Telegram error: {}, retrying connection.", te);
    //             RetryPolicy::WaitRetry(Duration::from_secs(30))
    //         }
    //         None => {
    //             log::error!("handle_error didn't match, real error {:?}", error);
    //             //handle_error didnt match, real error Io(Custom { kind: Other, error: StringError("failed to lookup address information: nodename nor servname provided, or not known") })
    //             RetryPolicy::ForwardError(error)
    //         }
    //     }
    // }

    pub fn establish_connection() -> DbConnPool {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = diesel::r2d2::ConnectionManager::new(database_url.clone());

        r2d2::Pool::builder()
            .min_idle(Some(1))
            .max_size(15)
            .build(manager)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }
}

pub struct BotState {
    commands_list: Vec<(String, String)>,
}

#[async_trait::async_trait]
impl Actor for BotActor {
    type Msg = BotActorMsg;
    type State = BotState;
    type Arguments = ();
    // (bot_name, bot, chan, lfg_chat): (String, Bot, ChannelRef<ActorUpdateMessage>, i64),
    // ) -> Self {
    // Self::new(&bot_name, bot, chan, lfg_chat)
    // }

    /// Register all bot commands and subscribe them to the system notification channel.
    async fn pre_start(
        &self,
        myself: ActorRef<Self>,
        (): Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let mut commands_list = vec![];

        macro_rules! new_command {
            ($T:ident) => {
                let Ok((cmd_actor, _handle)) = Actor::spawn_linked(
                    Some($T::actor_name()),
                    $T::new(
                        myself.clone(),
                        self.bot_name.clone(),
                        self.connection_pool.clone(),
                    ),
                    (),
                    myself.clone().into(),
                )
                .await
                else {
                    todo!();
                };
                commands_list.push(($T::prefix().into(), $T::description().into()));
                ractor::pg::join("raw-commands".into(), vec![cmd_actor.into()]);
            };
        }

        new_command!(ActivitiesCommand);
        new_command!(CancelCommand);
        new_command!(ChatidCommand);
        new_command!(D1weekCommand);
        new_command!(D2weekCommand);
        new_command!(EditCommand);
        new_command!(EditGuardianCommand);
        new_command!(HelpCommand);
        new_command!(InfoCommand);
        new_command!(JoinCommand);
        new_command!(LfgCommand);
        new_command!(ListCommand);
        new_command!(ManageCommand);
        new_command!(PsnCommand);
        new_command!(WhoisCommand);

        // Create reminder tasks actor
        let (reminders, _handle) = Actor::spawn_linked(
            Some("reminders".into()),
            ReminderActor::new(
                myself.clone(),
                self.lfg_chat_id,
                self.connection_pool.clone(),
            ),
            (),
            myself.clone().into(),
        )
        .await?;

        Ok(BotState { commands_list })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            BotActorMsg::SendMessage(message, chat_id, format, notify) => {
                log::debug!("SendMessage: {}", &message);
                let resp = self
                    .bot
                    .send_message(chat_id, message)
                    .disable_notification(match notify {
                        Notify::On => false,
                        Notify::Off => true,
                    })
                    .disable_web_page_preview(true);

                let mut resp = match format {
                    Format::Html => resp.parse_mode(ParseMode::Html),
                    Format::Markdown => resp.parse_mode(ParseMode::MarkdownV2),
                    Format::Plain => resp,
                };

                resp.send().await?;
                Ok(())
            }
            BotActorMsg::SendMessageReply(message, source, format, notify) => {
                log::debug!("SendMessageReply: {}", &message);

                let fut = self
                    .bot
                    .send_message(source.0.chat.id, message)
                    .reply_to_message_id(source.0.id)
                    .disable_notification(match notify {
                        Notify::On => false,
                        Notify::Off => true,
                    })
                    .disable_web_page_preview(true);

                let mut fut = match format {
                    Format::Html => fut.parse_mode(ParseMode::Html),
                    Format::Markdown => fut.parse_mode(ParseMode::MarkdownV2),
                    Format::Plain => fut,
                };

                fut.send().await?;
                Ok(())
            }
            BotActorMsg::ListCommands(source) => {
                log::debug!("ListCommands");

                let mut sorted_cmds = self.list_commands(state);
                sorted_cmds.sort_by_key(|v| v.0.clone());
                let reply = sorted_cmds.into_iter().fold(
                    // @todo TERA
                    "<b>Help</b> 🚑\nThese are the registered commands for this Bot:\n\n".into(),
                    |acc, pair| format!("{}{} — {}\n\n", acc, pair.0, pair.1),
                );

                cast!(
                    myself,
                    BotActorMsg::SendMessageReply(reply, source, Format::Html, Notify::Off)
                );
                Ok(())
            }
            BotActorMsg::RawCommand(message) => {
                for actor in ractor::pg::get_members(&String::from("raw-commands")).iter() {
                    actor.send_message(message)?;
                }
                Ok(())
            }
        }
    }
}
