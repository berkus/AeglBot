use crate::commands::{bot_command::BotCommand, validate_username};
use diesel::PgConnection;
use log::info;
use telegram_bot::{self, types::ParseMode, CanReplySendMessage};

pub struct LfgCommand;

impl LfgCommand {
    fn usage(api: &telegram_bot::Api, message: &telegram_bot::Message) {
        api.spawn(
            message
                .text_reply(
                    "LFG usage: /lfg <b>activity</b> timespec\n
            For a list of activity codes: /activities\n
            Example: /lfg kf tomorrow 23:00\n
            (NB: times are in MSK timezone by default)",
                ).parse_mode(ParseMode::Html),
        );
    }
}

impl BotCommand for LfgCommand {
    fn prefix() -> &'static str {
        "lfg"
    }

    fn description() -> &'static str {
        "Looking for group (if you want to create an event)"
    }

    fn execute(
        api: &telegram_bot::Api,
        message: &telegram_bot::Message,
        command: Option<String>,
        args: Option<String>,
        connection: &PgConnection,
    ) {
        info!("args are {:?}", args);

        if args.is_none() {
            return LfgCommand::usage(api, message);
        }

        // if (arguments.size < 2) {
        //     usage(absSender, chat)
        //     return
        // }

        if let Some(guardian) = validate_username(api, message, connection) {
            // val act = ActivityShortcut
            //     .find { ActivityShortcuts.name eq arguments[0] }
            //     .singleOrNull()

            // if (act == null) {
            //     sendReply(absSender, chat, "Activity ${arguments[0]} was not found. Use /activities for a list.")
            // } else {
            //     val startTime = parseTimeSpec(arguments.drop(1).joinToString(" "))

            //     val plannedActivity = PlannedActivity.new {
            //         author = dbUser
            //         activity = act.link
            //         start = startTime
            //         // set these using "/details id text" command
            //         details = ""
            //     }

            //     PlannedActivityMember.new {
            //         this.user = dbUser
            //         this.activity = plannedActivity
            //     }

            //     sendReply(absSender, chat, // Todo: always post to lfg chat?
            //         "${dbUser.formatName()} is looking for ${act.link.formatName()} group ${formatStartTime(startTime)}\n"
            //         +plannedActivity.joinPrompt()+"\n"
            //         +"Use `/details ${plannedActivity.id} description` to specify more details about the event.")
            // }
        }
    }
}
