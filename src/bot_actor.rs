use {
    crate::{
        commands::*, services::reminder_actor::ReminderActor, BotCommand, DbConnPool, NamedActor,
    },
    dotenv::dotenv,
    ractor::{cast, Actor, ActorProcessingErr, ActorRef},
    std::{env, fmt::Formatter},
    teloxide::{
        prelude::*,
        types::{ChatId, ParseMode},
    },
};

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

pub async fn send_message(
    bot: Bot,
    message: Message,
    chat_id: ChatId,
    format: Format,
    notify: Notify,
) -> Result<()> {
    log::debug!("send_message: {}", &message);
    let resp = bot
        .send_message(chat_id, message)
        .disable_notification(match notify {
            Notify::On => false,
            Notify::Off => true,
        })
        .disable_web_page_preview(true);

    let mut resp = match format {
        Format::Html => resp.parse_mode(ParseMode::Html),
        Format::Markdown => resp.parse_mode(ParseMode::MarkdownV2),
        Format::Plain => resp,
    };

    resp.send().await?;
    Ok(())
}

pub async fn send_message_reply(
    bot: Bot,
    message: Message,
    source: Message,
    format: Format,
    notify: Notify,
) -> Result<()> {
    log::debug!("send_message_reply: {}", &message);

    let fut = bot
        .send_message(source.0.chat.id, message)
        .reply_to_message_id(source.0.id)
        .disable_notification(match notify {
            Notify::On => false,
            Notify::Off => true,
        })
        .disable_web_page_preview(true);

    let mut fut = match format {
        Format::Html => fut.parse_mode(ParseMode::Html),
        Format::Markdown => fut.parse_mode(ParseMode::MarkdownV2),
        Format::Plain => fut,
    };

    fut.send().await?;
    Ok(())
}

pub fn list_commands(&self, state: &mut <Self as Actor>::State) -> Vec<(String, String)> {
    state.commands_list.clone()
    // let mut sorted_cmds = self.list_commands(state);
    // sorted_cmds.sort_by_key(|v| v.0.clone());
    // let reply = sorted_cmds.into_iter().fold(
    //     // @todo TERA
    //     "<b>Help</b> 🚑\nThese are the registered commands for this Bot:\n\n".into(),
    //     |acc, pair| format!("{}{} — {}\n\n", acc, pair.0, pair.1),
    // );
    //
    // cast!(
    //                 myself,
    //                 BotActorMsg::SendMessageReply(reply, source, Format::Html, Notify::Off)
    //             );
}

/// Register all bot commands and subscribe them to the system notification channel.
async fn pre_start(
    &self,
    myself: ActorRef<Self>,
    (): Self::Arguments,
) -> Result<Self::State, ActorProcessingErr> {
    let mut commands_list = vec![];

    macro_rules! new_command {
            ($T:ident) => {
                let Ok((cmd_actor, _handle)) = Actor::spawn_linked(
                    Some($T::actor_name()),
                    $T::new(myself.clone(), self.bot_name.clone(), self.connection_pool.clone()),
                    (),
                    myself.clone().into(),
                )
                .await else { todo!(); };
                commands_list
                    .push(($T::prefix().into(), $T::description().into()));
                ractor::pg::join("raw-commands".into(), vec![cmd_actor.into()]);
            };
        }

    new_command!(ActivitiesCommand);
    // new_command!(CancelCommand);
    new_command!(ChatidCommand);
    new_command!(D1weekCommand);
    new_command!(D2weekCommand);
    new_command!(EditCommand);
    new_command!(EditGuardianCommand);
    new_command!(HelpCommand);
    new_command!(InfoCommand);
    new_command!(JoinCommand);
    new_command!(LfgCommand);
    new_command!(ListCommand);
    new_command!(ManageCommand);
    new_command!(PsnCommand);
    new_command!(WhoisCommand);

    // @todo this should go to bot.rs, does teloxide have anything for scheduling? prolly not.
    // Create reminder tasks actor
    let (reminders, _handle) = Actor::spawn_linked(
        Some("reminders".into()),
        ReminderActor::new(
            myself.clone(),
            self.lfg_chat_id,
            self.connection_pool.clone(),
        ),
        (),
        myself.clone().into(),
    )
    .await?;

    Ok(BotState { commands_list })
}

async fn handle(
    &self,
    myself: ActorRef<Self>,
    message: Self::Msg,
    state: &mut Self::State,
) -> Result<(), ActorProcessingErr> {
    match message {
        BotActorMsg::ListCommands(source) => {
            log::debug!("ListCommands");

            Ok(())
        }
    }
}
