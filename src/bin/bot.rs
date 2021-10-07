// Async Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (yeah, i'd prefer native, although there are ways to use natty through jlink
// or take python equivalent from https://dateparser.readthedocs.io/en/latest/)
#![feature(box_syntax, nll)]

use {
    aegl_bot::{
        commands::*,
        datetime::{d2_reset_time, reference_date, start_at_time, start_at_weekday_time},
        services::{
            reminder_actor::{
                ReminderActor, ScheduleNextDay, ScheduleNextMinute, ScheduleNextWeek,
            },
            *,
        },
        BotMenu, UpdateMessage,
    },
    dotenv::dotenv,
    futures::{Future, Stream},
    // riker::prelude::*, doesn't work here!
    riker::{
        actor::Tell,
        actors::{channel, ActorRefFactory, ActorSystem, ChannelRef, Publish, Subscribe},
    },
    std::{env, time::Instant},
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

    // TimeZone.setDefault(TimeZone.getTimeZone(config.getString("bot.timezone")))
    let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
    let lfg_chat = env::var("BOT_LFG_CHAT_ID")
        .expect("BOT_LFG_CHAT_ID must be set")
        .parse::<i64>()
        .expect("BOT_LFG_CHAT_ID must be a valid telegram chat id");

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let sys = ActorSystem::new().unwrap();

    let bot = sys.actor_of_args::<BotMenu, _>("bot", (bot_name, token))?;

    let chan: ChannelRef<UpdateMessage> = channel("commands", &sys).unwrap();

    fn new_command<T>(sys: ActorSystem, chan: &ChannelRef<UpdateMessage>) /*->anyhow::Result<()>*/
    {
        let cmd = sys.actor_of::<T>()?;
        chan.tell(
            Subscribe {
                actor: cmd,
                topic: "raw-commands".into(),
            },
            None,
        );
    }

    // new_command::<ActivitiesCommand>();
    // new_command::<CancelCommand>();
    // new_command::<D2weekCommand>();
    // new_command::<D1weekCommand>();
    // new_command::<EditCommand>();
    // new_command::<EditGuardianCommand>();
    // new_command::<HelpCommand>();
    new_command::<InfoCommand>(sys, &chan);
    // new_command::<JoinCommand>();
    // new_command::<LfgCommand>();
    // new_command::<ListCommand>();
    // new_command::<ManageCommand>();
    // new_command::<PsnCommand>();
    // new_command::<WhoisCommand>();

    // Reminder tasks
    let reminders = sys
        .actor_of_args::<ReminderActor, _>("reminders", (bot.clone(), lfg_chat))
        .unwrap();
    // Schedule first run, the actor handler will reschedule.
    reminders.tell(ScheduleNextMinute, None);
    reminders.tell(ScheduleNextDay, None);
    reminders.tell(ScheduleNextWeek, None);

    teloxide::repl(bot.bot.clone(), |message| async {
        chan.tell(
            Publish {
                msg: message,
                topic: "raw-commands".into(),
            },
            None,
        );
        ResponseResult::<()>::Ok(())
    })
    .await;
}
