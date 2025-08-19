use {
    crate::{
        actors::reminder_actor::{
            ReminderActor, ScheduleNextDay, ScheduleNextMinute, ScheduleNextWeek,
        },
        commands::*,
        BotCommand,
    },
    kameo::{
        actor::ActorRef,
        error::Infallible,
        message::{Context, Message},
        Actor,
    },
    sea_orm::DatabaseConnection,
    std::fmt::Formatter,
    teloxide::{
        prelude::*,
        types::{ChatId, ParseMode},
    },
    tokio::sync::broadcast,
};

pub struct BotActor {
    pub bot: Bot,
    bot_name: String,
    lfg_chat_id: i64,
    update_sender: broadcast::Sender<ActorUpdateMessage>,
    connection_pool: DatabaseConnection,
    commands_list: Vec<(String, String)>,
}

unsafe impl Send for BotActor {}

impl std::fmt::Debug for BotActor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BotActor")
    }
}

#[derive(Debug, Clone)]
pub struct ActorUpdateMessage {
    pub requester: Bot,
    pub update: teloxide::types::Message,
}

impl ActorUpdateMessage {
    pub fn new(requester: Bot, update: teloxide::types::Message) -> Self {
        Self { requester, update }
    }
}

impl BotActor {
    // Public API

    pub async fn new(
        name: &str,
        bot: Bot,
        update_sender: broadcast::Sender<ActorUpdateMessage>,
        lfg_chat_id: i64,
    ) -> Self {
        let connection_pool = crate::establish_db_connection().await.unwrap();
        BotActor {
            bot,
            bot_name: name.to_string(),
            lfg_chat_id,
            update_sender,
            connection_pool,
            commands_list: vec![],
        }
    }

    pub fn list_commands(&self) -> Vec<(String, String)> {
        self.commands_list.clone()
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
}

use crate::commands::match_command;

impl Actor for BotActor {
    type Args = Self;
    type Error = Infallible;

    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        macro_rules! new_command {
            ($T:ident, $args:expr) => {
                let cmd = $T::spawn($T::new(
                    actor_ref.clone(),
                    $args.bot_name.clone(),
                    $args.connection_pool.clone(),
                ));
                $args
                    .commands_list
                    .push(($T::prefix().into(), $T::description().into()));

                // Subscribe to updates
                let mut update_receiver = $args.update_sender.subscribe();
                let cmd_clone = cmd.clone();
                let bot_name = $args.bot_name.clone();
                tokio::spawn(async move {
                    while let Ok(msg) = update_receiver.recv().await {
                        if let (Some(_), _) =
                            match_command(msg.update.text(), $T::prefix(), &bot_name)
                        {
                            let _ = cmd_clone.tell(msg).await;
                        }
                    }
                });
            };
        }

        let mut bot_actor = args;

        new_command!(ActivitiesCommand, bot_actor);
        new_command!(CancelCommand, bot_actor);
        new_command!(ChatidCommand, bot_actor);
        new_command!(D1weekCommand, bot_actor);
        new_command!(D2weekCommand, bot_actor);
        new_command!(EditCommand, bot_actor);
        new_command!(EditGuardianCommand, bot_actor);
        new_command!(HelpCommand, bot_actor);
        new_command!(JoinCommand, bot_actor);
        new_command!(LfgCommand, bot_actor);
        new_command!(ListCommand, bot_actor);
        new_command!(ManageCommand, bot_actor);
        new_command!(PsnCommand, bot_actor);
        new_command!(UptimeCommand, bot_actor);
        new_command!(WhoisCommand, bot_actor);

        // Create reminder tasks actor
        let reminders = ReminderActor::spawn(ReminderActor::new(
            actor_ref.clone(),
            bot_actor.lfg_chat_id,
            bot_actor.connection_pool.clone(),
        ));

        // Schedule first run, the actor handler will reschedule.
        let _ = reminders.tell(ScheduleNextMinute).await;
        let _ = reminders.tell(ScheduleNextDay).await;
        let _ = reminders.tell(ScheduleNextWeek).await;

        Ok(bot_actor)
    }
}

impl BotActor {
    pub async fn create(
        bot_name: String,
        bot: Bot,
        update_sender: broadcast::Sender<ActorUpdateMessage>,
        lfg_chat: i64,
    ) -> Self {
        Self::new(&bot_name, bot, update_sender, lfg_chat).await
    }
}

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

#[derive(Clone, Debug)]
pub struct SendMessage(pub String, pub ChatId, pub Format, pub Notify);

#[derive(Clone, Debug)]
pub struct SendMessageReply(pub String, pub ActorUpdateMessage, pub Format, pub Notify);

#[derive(Clone, Debug)]
pub struct ListCommands(pub ActorUpdateMessage);

impl Message<SendMessage> for BotActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: SendMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        log::debug!("SendMessage: {}", &msg.0);
        let resp = self
            .bot
            .send_message(msg.1, msg.0)
            .disable_notification(match msg.3 {
                Notify::On => false,
                Notify::Off => true,
            })
            .link_preview_options(teloxide::types::LinkPreviewOptions {
                is_disabled: true,
                url: None,
                prefer_small_media: false,
                prefer_large_media: false,
                show_above_text: false,
            });

        let resp = match msg.2 {
            Format::Html => resp.parse_mode(ParseMode::Html),
            Format::Markdown => resp.parse_mode(ParseMode::MarkdownV2),
            Format::Plain => resp,
        };

        let _ = resp.send().await;
    }
}

impl Message<SendMessageReply> for BotActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: SendMessageReply,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        log::debug!("SendMessageReply: {}", &msg.0);
        let message = msg.1;

        let fut = self
            .bot
            .send_message(message.update.chat.id, msg.0)
            .reply_parameters(teloxide::types::ReplyParameters::new(message.update.id))
            .disable_notification(match msg.3 {
                Notify::On => false,
                Notify::Off => true,
            })
            .link_preview_options(teloxide::types::LinkPreviewOptions {
                is_disabled: true,
                url: None,
                prefer_small_media: false,
                prefer_large_media: false,
                show_above_text: false,
            });

        let fut = match msg.2 {
            Format::Html => fut.parse_mode(ParseMode::Html),
            Format::Markdown => fut.parse_mode(ParseMode::MarkdownV2),
            Format::Plain => fut,
        };

        let _ = fut.send().await;
    }
}

impl Message<ListCommands> for BotActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ListCommands,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        log::debug!("ListCommands");
        let message = msg.0;

        let mut sorted_cmds = self.list_commands();
        sorted_cmds.sort_by_key(|v| v.0.clone());
        let reply = sorted_cmds.into_iter().fold(
            "<b>Help</b> 🚑\nThese are the registered commands for this Bot:\n\n".into(),
            |acc, pair| format!("{}{} — {}\n\n", acc, pair.0, pair.1),
        );

        let _ = ctx
            .actor_ref()
            .tell(SendMessageReply(reply, message, Format::Html, Notify::Off))
            .try_send(); // @todo use unbounded mailbox for bot_actor? prolly not
    }
}
