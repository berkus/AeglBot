// Async Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (yeah, i'd prefer native, although there are ways to use natty through jlink
// or take python equivalent from https://dateparser.readthedocs.io/en/latest/)
#![feature(box_syntax)]
#![feature(associated_type_bounds)]

use {
    aegl_bot::bot_actor::{BotActor, BotActorMsg, CommandMsg},
    dotenv::dotenv,
    ractor::{cast, Actor, ActorProcessingErr, ActorRef},
    std::env,
    teloxide::{
        dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
        prelude::*,
        types::MessageId,
        utils::command::BotCommands,
    },
};

fn setup_logging() -> Result<(), fern::InitError> {
    use fern::colors::{Color, ColoredLevelConfig};

    // Color setup from fern examples
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);
    let colors_level = colors_line.info(Color::Green);

    let console_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}{date}[{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout());

    let file_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false) // don't overwrite log file each run
                .open(format!(
                    "logs/bot-{}.log",
                    chrono::Local::now().format("%Y%m%d-%H%M%S")
                ))?,
        );

    fern::Dispatch::new()
        .chain(console_config)
        .chain(file_config)
        .apply()?;

    Ok(())
}

#[derive(BotCommands, PartialEq, Debug, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    // Start,
    // #[command(description = "List available activity shortcuts")]
    // Activities,
    #[command(description = "Leave joined activity")]
    Cancel(i32),
    // #[command(description = "Figure out the numeric chat ID")]
    // ChatId,
    // #[command(description = "Show current Destiny 2 week")]
    // D2Week,
    // #[command(description = "Show current Destiny 1 week")]
    // DWeek,
    // #[command(description = "Edit existing activity")]
    // Edit,
    // #[command(description = "Edit information about registered guardians")]
    // EditGuar,
    // #[command(description = "List available commands")]
    // Help,
    // #[command(description = "Show bot info and statistics")]
    // Info,
    // #[command(description = "Join existing activity from the list")]
    // Join,
    // #[command(description = "Create a new Looking For Group event")]
    // LFG,
    // #[command(description = "List current events")]
    // List,
    // #[command(description = "Manage bot users (admin-only)")]
    // Manage,
    // #[command(description = "Link your telegram user to PSN")]
    // PSN,
    // #[command(description = "Query telegram or PSN id")]
    // WhoIs,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    setup_logging().expect("failed to initialize logging");

    // TimeZone.setDefault(TimeZone.getTimeZone(config.getString("bot.timezone")))
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
    let lfg_chat = env::var("BOT_LFG_CHAT_ID")
        .expect("BOT_LFG_CHAT_ID must be set")
        .parse::<i64>()
        .expect("BOT_LFG_CHAT_ID must be a valid telegram chat id");

    let tgbot = Bot::new(token); // Bot::from_env?

    let parameters = ConfigParameters {
        bot_name: String,
        lfg_chat_id: i64,
        connection_pool: establish_connection(),
        bot_maintainer: UserId(0), // Paste your ID to run this bot.
        maintainer_username: None,
    };

    Dispatcher::builder(tgbot, build_handler())
        // .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .dependencies(dptree::deps![parameters]) // no more capture, pass by argument
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher", // log to rollbar etc
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

struct ConfigParameters {}

impl ConfigParameters {
    pub fn connection(&self) -> BotConnection {
        self.connection_pool.get().unwrap()
    }
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    // ReceiveFullName,
    // ReceiveProductChoice {
    //     full_name: String,
    // },
}

fn build_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![State::Start].endpoint(command_handler));

    // .branch(
    //     case![State::Start]
    //         .branch(case![Command::Help].endpoint(help))
    //         .branch(case![Command::Start].endpoint(start)),
    // )
    // .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        // .branch(case![State::ReceiveFullName].endpoint(receive_full_name))
        .branch(dptree::endpoint(invalid_state));

    // let callback_query_handler = Update::filter_callback_query().branch(
    //     case![State::ReceiveProductChoice { full_name }].endpoint(receive_product_selection),
    // );

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
    // .branch(callback_query_handler)
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    todo!()
}
async fn help(bot: Bot, msg: Message) -> HandlerResult {
    todo!()
}
async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    todo!()
}
async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    todo!()
}
async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    todo!()
}
async fn receive_product_selection(
    bot: Bot,
    dialogue: MyDialogue,
    full_name: String, // Available from `State::ReceiveProductChoice`.
    q: CallbackQuery,
) -> HandlerResult {
    todo!()
}

async fn command_handler(
    config: ConfigParameters,
    bot: Bot,
    me: teloxide::types::Me,
    message: Message,
    command: Command,
) -> HandlerResult {
    let text = match command {
        Command::Cancel(id) => cancel_command(),
    };
    let MessageId(id) = message.id;
    log::trace!("Processing message {}", id);
    Ok(())
}

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
