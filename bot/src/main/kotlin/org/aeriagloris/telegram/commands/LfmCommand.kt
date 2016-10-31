package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore

class LfmCommand(val store: JdbcStore) : ExtendedCommand("lfm", "Looking for member (if you're fireteam looking to fill some positions)")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        sendReply(absSender, chat, "NOT IMPLEMENTED")
        //sendReply(absSender, chat, "Looking for member")
    }
}
