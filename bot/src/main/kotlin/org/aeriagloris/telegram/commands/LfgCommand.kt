package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.bots.commands.BotCommand
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore

class LfgCommand(val store: JdbcStore) : BotCommand("lfg", "Looking for group (if you look for a fireteam)")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        val answer = SendMessage()
        answer.setChatId(chat.getId().toString())
        answer.setText("Looking for group")

        try {
            absSender.sendMessage(answer)
        } catch (e: TelegramApiException) {
            BotLogger.error("LFGCOMMAND", e)
        }
    }
}
