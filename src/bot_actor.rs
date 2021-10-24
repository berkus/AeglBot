use {
    crate::{commands::*, DbConnPool, NamedActor},
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
#[actor(SendMessage, SendMessageReply)]
pub struct BotActor {
    pub bot: Bot,
    bot_name: String,
    update_channel: ChannelRef<ActorUpdateMessage>,
    connection_pool: DbConnPool,
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

    pub fn new(name: &str, bot: Bot, chan: ChannelRef<ActorUpdateMessage>) -> Self {
        BotActor {
            bot,
            bot_name: name.to_string(),
            update_channel: chan,
            connection_pool: Self::establish_connection(),
        }
    }

    // pub fn register_catchall(cmd: Box<BotCommand>) {}

    // Insert into commands while maintaining certain property:
    // - if command is a prefix of another inserted command, it must be inserted after
    //   that command.
    // - otherwise the command is inserted to the very beginning of vector.
    // This allows correct parsing order fof commands that are prefixes of another command.
    // pub fn register_command(&mut self, cmd: Box<dyn BotCommand>) {
    //     let mut insertion_index = 0;
    //     for (idx, command) in self.commands.read().unwrap().iter().enumerate() {
    //         if command.prefix().starts_with(cmd.prefix()) {
    //             insertion_index = idx + 1;
    //         }
    //     }
    //
    //     self.commands.write().unwrap().insert(insertion_index, cmd);
    // }

    // pub fn list_commands(&self) -> Vec<(String, String)> {
    //     self.commands
    //         .read()
    //         .unwrap()
    //         .iter()
    //         .fold(vec![], |mut acc, cmd| {
    //             acc.push((cmd.prefix().to_string(), cmd.description().to_string()));
    //             acc
    //         })
    // }

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

    // @todo Make this a message processor in Actor
    // @todo Send commands as messages too? Need dynamic command definition then...
    // pub fn process_message(&self, message: UpdateMessage) {
    //     let message = &message;
    //     for cmd in self.commands.read().unwrap().iter() {
    //         if let (Some(cmdname), text) =
    //             Self::match_command(message, cmd.prefix(), &self.bot_name)
    //         {
    //             return cmd.execute(&self, message, Some(cmdname), text);
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
        // new_command::<EditCommand>();
        // new_command::<EditGuardianCommand>();
        // new_command::<HelpCommand>();
        new_command!(InfoCommand);
        new_command!(JoinCommand);
        new_command!(LfgCommand);
        new_command!(ListCommand);
        // new_command::<ManageCommand>();
        new_command!(PsnCommand);
        new_command!(WhoisCommand);
    }

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}

impl ActorFactoryArgs<(String, Bot, ChannelRef<ActorUpdateMessage>)> for BotActor {
    fn create_args((bot_name, bot, chan): (String, Bot, ChannelRef<ActorUpdateMessage>)) -> Self {
        Self::new(&bot_name, bot, chan)
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
