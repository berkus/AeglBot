package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore

class RaidCommand(val store: JdbcStore) : ExtendedCommand("raid", "Set up a raid party")
{
    // Arguments
    // [0] - raid id (wrath, kf, etc)
    // [1] - normal/hard
    // [2] - when (date/time)
    // 
    // Inline mode:
    // @AeglBot raid [same args with helper selectors]
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        sendReply(absSender, chat, "NOT IMPLEMENTED")
        //sendReply(absSender, chat, "You are raiding!")
    }
}
