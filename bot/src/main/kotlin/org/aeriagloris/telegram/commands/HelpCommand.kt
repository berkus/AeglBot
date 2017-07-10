package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.bots.commandbot.commands.BotCommand
import org.telegram.telegrambots.bots.commandbot.commands.ICommandRegistry
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.telegram.services.Emoji

class HelpCommand(val commandRegistry: ICommandRegistry)
    : ExtendedCommand("help", "List available commands")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        val helpMessageBuilder = StringBuilder("<b>Help</b> " + Emoji.AMBULANCE + "\n")
        helpMessageBuilder.append("These are the registered commands for this Bot:\n\n")

        commandRegistry.getRegisteredCommands().forEach { botCommand: BotCommand ->
            helpMessageBuilder.append(botCommand.toString()).append("\n\n")
        }

        sendReply(absSender, chat, helpMessageBuilder.toString(), true)
    }
}
