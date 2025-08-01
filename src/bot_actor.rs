use {
    crate::{
        commands::*,
        services::reminder_actor::{
            ReminderActor, ScheduleNextDay, ScheduleNextMinute, ScheduleNextWeek,
        },
        BotCommand, DbConnPool, NamedActor,
    },
    dotenv::dotenv,
    riker::actors::{
        actor, Actor, ActorFactoryArgs, ActorRefFactory, BasicActorRef, ChannelRef, Context,
        Receive, Sender, Subscribe, Tell,
    },
    std::{env, fmt::Formatter},
    teloxide::{
        prelude::*,
        types::{ChatId, ParseMode},
    },
};

#[derive(Clone)]
#[actor(SendMessage, SendMessageReply, ListCommands)]
pub struct BotActor {
    pub bot: Bot,
    bot_name: String,
    lfg_chat_id: i64,
    update_channel: ChannelRef<ActorUpdateMessage>,
    connection_pool: DbConnPool,
    commands_list: Vec<(String, String)>,
}

unsafe impl Send for BotActor {}

impl std::fmt::Debug for BotActor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BotActor")
    }
}

pub type UpdateMessage = UpdateWithCx<Bot, Message>;
pub type ActorUpdateMessage = ActorUpdateWithCx<Bot, Message>;

// Manually derived version of UpdateWithCx<_, _>
#[derive(Debug, Clone)]
pub struct ActorUpdateWithCx<R, Upd> {
    pub requester: R,
    pub update: Upd,
}

impl From<UpdateMessage> for ActorUpdateMessage {
    fn from(m: UpdateMessage) -> Self {
        Self {
            requester: m.requester,
            update: m.update,
        }
    }
}

impl BotActor {
    // Public API

    pub fn new(
        name: &str,
        bot: Bot,
        chan: ChannelRef<ActorUpdateMessage>,
        lfg_chat_id: i64,
    ) -> Self {
        BotActor {
            bot,
            bot_name: name.to_string(),
            lfg_chat_id,
            update_channel: chan,
            connection_pool: Self::establish_connection(),
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

impl Actor for BotActor {
    type Msg = BotActorMsg;

    /// Register all bot commands and subscribe them to the system notification channel.
    fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
        macro_rules! new_command {
            ($T:ident) => {
                let cmd = ctx
                    .actor_of_args::<$T, _>(&$T::actor_name(), (ctx.myself().clone(), self.bot_name.clone(), self.connection_pool.clone()))
                    .unwrap(); // FIXME: panics in pre_start do not cause actor restart, so this is faulty!
                self.commands_list.push(($T::prefix().into(), $T::description().into()));
                self.update_channel.tell(
                    Subscribe {
                        actor: Box::new(cmd),
                        topic: "raw-commands".into(),
                    },
                    None,
                );
            }
        }

        new_command!(ActivitiesCommand);
        new_command!(CancelCommand);
        new_command!(ChatidCommand);
        new_command!(D1weekCommand);
        new_command!(D2weekCommand);
        new_command!(EditCommand);
        new_command!(EditGuardianCommand);
        new_command!(HelpCommand);
        new_command!(JoinCommand);
        new_command!(LfgCommand);
        new_command!(ListCommand);
        new_command!(ManageCommand);
        new_command!(PsnCommand);
        new_command!(UptimeCommand);
        new_command!(WhoisCommand);

        // Create reminder tasks actor
        let reminders = ctx
            .actor_of_args::<ReminderActor, _>(
                "reminders",
                (ctx.myself(), self.lfg_chat_id, self.connection_pool.clone()),
            )
            .unwrap();
        // Schedule first run, the actor handler will reschedule.
        reminders.tell(ScheduleNextMinute, None);
        reminders.tell(ScheduleNextDay, None);
        reminders.tell(ScheduleNextWeek, None);
    }

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}

impl ActorFactoryArgs<(String, Bot, ChannelRef<ActorUpdateMessage>, i64)> for BotActor {
    fn create_args(
        (bot_name, bot, chan, lfg_chat): (String, Bot, ChannelRef<ActorUpdateMessage>, i64),
    ) -> Self {
        Self::new(&bot_name, bot, chan, lfg_chat)
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

impl Receive<SendMessage> for BotActor {
    type Msg = BotActorMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: SendMessage, _sender: Sender) {
        log::debug!("SendMessage: {}", &msg.0);
        let resp = self
            .bot
            .send_message(msg.1, msg.0)
            .disable_notification(match msg.3 {
                Notify::On => false,
                Notify::Off => true,
            })
            .disable_web_page_preview(true);

        let resp = match msg.2 {
            Format::Html => resp.parse_mode(ParseMode::Html),
            Format::Markdown => resp.parse_mode(ParseMode::MarkdownV2),
            Format::Plain => resp,
        };

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(resp.send()).unwrap();
    }
}

impl Receive<SendMessageReply> for BotActor {
    type Msg = BotActorMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: SendMessageReply, _sender: Sender) {
        log::debug!("SendMessageReply: {}", &msg.0);
        let message = msg.1;

        let fut = self
            .bot
            .send_message(message.update.chat_id(), msg.0)
            .reply_to_message_id(message.update.id)
            .disable_notification(match msg.3 {
                Notify::On => false,
                Notify::Off => true,
            })
            .disable_web_page_preview(true);

        let fut = match msg.2 {
            Format::Html => fut.parse_mode(ParseMode::Html),
            Format::Markdown => fut.parse_mode(ParseMode::MarkdownV2),
            Format::Plain => fut,
        };

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(fut.send()).unwrap();
    }
}

impl Receive<ListCommands> for BotActor {
    type Msg = BotActorMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: ListCommands, _sender: Sender) {
        log::debug!("ListCommands");
        let message = msg.0;

        let mut sorted_cmds = self.list_commands();
        sorted_cmds.sort_by_key(|v| v.0.clone());
        let reply = sorted_cmds.into_iter().fold(
            "<b>Help</b> 🚑\nThese are the registered commands for this Bot:\n\n".into(),
            |acc, pair| format!("{}{} — {}\n\n", acc, pair.0, pair.1),
        );

        ctx.myself.tell(
            SendMessageReply(reply, message, Format::Html, Notify::Off),
            None,
        );
    }
}
