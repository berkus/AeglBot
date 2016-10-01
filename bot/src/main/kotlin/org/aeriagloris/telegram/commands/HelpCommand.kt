package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.bots.commands.BotCommand
import org.telegram.telegrambots.bots.commands.ICommandRegistry
import org.telegram.telegrambots.logging.BotLogger

class HelpCommand(val commandRegistry: ICommandRegistry) : BotCommand("help", "List available commands")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        val helpMessageBuilder = StringBuilder("<b>Help</b>\n")
        helpMessageBuilder.append("These are the registered commands for this Bot:\n\n")

        commandRegistry.getRegisteredCommands().forEach { botCommand: BotCommand ->
            helpMessageBuilder.append(botCommand.toString()).append("\n\n")
        }

        val helpMessage = SendMessage()
        helpMessage.setChatId(chat.getId().toString())
        helpMessage.enableHtml(true)
        helpMessage.setText(helpMessageBuilder.toString())

        try {
            absSender.sendMessage(helpMessage)
        } catch (e: TelegramApiException) {
            BotLogger.error("HELPCOMMAND", e)
        }
    }
}
