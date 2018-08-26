use diesel::PgConnection;

pub trait BotCommand {
    fn prefix() -> &'static str;
    fn description() -> &'static str;
    fn execute(
        api: &telegram_bot::Api,
        message: &telegram_bot::Message,
        command: Option<String>,
        text: Option<String>,
        connection: &PgConnection,
    );
}
