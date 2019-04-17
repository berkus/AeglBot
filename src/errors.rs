/// Implement failure Fail for various types used in the bot
/// @todo use anyhow
use failure::Error;

#[derive(Debug, Fail)]
enum BotError {
    DbError(diesel::result::Error),
}
