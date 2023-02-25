// Async Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (There are now several rust impls including https://lib.rs/crates/two_timer and https://lib.rs/crates/intervalle)

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
    Start,
    // Activities,
    Cancel,
    // ChatId,
    // D2Week,
    // DWeek,
    // Edit,
    // EditGuar,
    Help,
    // Info,
    // Join,
    // LFG,
    #[command(description = "List current events")]
    List,
    // Manage,
    // PSN,
    // WhoIs,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    setup_logging().expect("failed to initialize logging");

    aegl_bot::datetime::bot_start_time(); // Mark start timestamp

    // TimeZone.setDefault(TimeZone.getTimeZone(config.getString("bot.timezone")))
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
    let lfg_chat = env::var("BOT_LFG_CHAT_ID")
        .expect("BOT_LFG_CHAT_ID must be set")
        .parse::<i64>()
        .expect("BOT_LFG_CHAT_ID must be a valid telegram chat id");

    let tgbot = Bot::new(token); // Bot::from_env?

    let bot_actor = BotActor::new(&bot_name, tgbot.clone(), lfg_chat);

    let (actor, _handle) = Actor::spawn(
        None,
        bot_actor,
        (), //?
    )
    .await
    .expect("Couldn't start the bot");

    Dispatcher::builder(tgbot, build_handler())
        // .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .dependencies(dptree::deps![actor]) // no more capture, pass by argument
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
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

    let command_handler =
        teloxide::filter_command::<Command, _>().branch(case![State::Start].endpoint(handler));

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
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

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

async fn handler(bot: Bot, message: Message, actor: ActorRef<BotActor>) -> HandlerResult {
    let MessageId(id) = message.id;
    log::trace!("Processing message {}", id);
    actor.send_message(BotActorMsg::RawCommand(message))?;
    Ok(())
}
