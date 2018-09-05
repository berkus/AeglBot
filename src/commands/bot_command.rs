use crate::DbConnection;

pub trait BotCommand {
    fn prefix() -> &'static str;
    fn description() -> &'static str;
    fn execute(
        bot: &telebot::RcBot,
        message: telebot::objects::Message,
        command: Option<String>,
        text: Option<String>,
        connection: &DbConnection,
    );
}
