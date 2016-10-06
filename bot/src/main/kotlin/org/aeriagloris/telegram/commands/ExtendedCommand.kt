package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.bots.commands.BotCommand
import org.telegram.telegrambots.logging.BotLogger

abstract class ExtendedCommand(tag: String, text: String) : BotCommand(tag, text)
{
    fun sendReply(absSender: AbsSender, chat: Chat, message: String, isHtml: Boolean = false) {
        val answer = SendMessage()
        answer.setChatId(chat.getId().toString())
        answer.enableHtml(isHtml)
        answer.setText(message)

        try {
            absSender.sendMessage(answer)
        } catch (e: TelegramApiException) {
            BotLogger.error("COMMAND", e)
        }
    }
}
