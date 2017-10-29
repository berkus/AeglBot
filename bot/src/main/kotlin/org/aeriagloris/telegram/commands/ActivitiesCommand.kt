package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.bots.commandbot.commands.BotCommand
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.telegram.services.Emoji
import org.aeriagloris.persistence.JdbcStore
import org.jetbrains.exposed.sql.transactions.transaction
import org.aeriagloris.persistence.schema.*
import org.jetbrains.exposed.sql.*

class ActivitiesCommand(val store: JdbcStore)
    : ExtendedCommand("activities", "List available activity shortcuts")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        transaction {
            var text = "Activities: use a short name:\n";
            val games = ActivityShortcuts.slice(ActivityShortcuts.game).selectAll().withDistinct().toList()
                .map { game -> game[ActivityShortcuts.game] }.sorted()

            for (game in games) {
                text += "*** <b>${game}</b>:\n" +
                    ActivityShortcut.find { ActivityShortcuts.game eq game }.toList().sortedBy { ActivityShortcuts.name }.map { act ->
                        "<b>${act.name}</b>\t${act.link.formatName()}"
                    }.joinToString("\n") + "\n"
            }

            sendReply(absSender, chat, text, true)
        }
    }
}
