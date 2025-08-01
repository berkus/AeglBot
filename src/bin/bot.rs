// Async Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (There are now several rust impls including https://lib.rs/crates/two_timer and https://lib.rs/crates/intervalle)

use {
    aegl_bot::bot_actor::{ActorUpdateMessage, BotActor, UpdateMessage},
    dotenv::dotenv,
    // riker::prelude::*, doesn't work here!
    riker::actors::{channel, ActorRefFactory, ActorSystem, ChannelRef, Publish, Tell},
    std::env,
    teloxide::{prelude::*, requests::ResponseResult},
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

#[tokio::main]
async fn main() {
    dotenv().ok();
    setup_logging().expect("failed to initialize logging");

    aegl_bot::datetime::bot_start_time(); // Mark start timestamp

    // TimeZone.setDefault(TimeZone.getTimeZone(config.getString("bot.timezone")))
    let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
    let lfg_chat = env::var("BOT_LFG_CHAT_ID")
        .expect("BOT_LFG_CHAT_ID must be set")
        .parse::<i64>()
        .expect("BOT_LFG_CHAT_ID must be a valid telegram chat id");

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let sys = ActorSystem::new().unwrap();

    let tgbot = Bot::new(token);

    let chan: ChannelRef<ActorUpdateMessage> = channel("commands", &sys).unwrap();

    let _bot_ref = sys
        .actor_of_args::<BotActor, _>("bot", (bot_name, tgbot.clone(), chan.clone(), lfg_chat))
        .expect("Couldn't start the bot");

    teloxide::repl(tgbot.clone(), move |message: UpdateMessage| {
        let chan = chan.clone();
        async move {
            log::debug!("Processing message {}", message.update.id);
            chan.tell(
                Publish {
                    msg: message.into(),
                    topic: "raw-commands".into(),
                },
                None,
            );
            ResponseResult::<()>::Ok(())
        }
    })
    .await;
}
