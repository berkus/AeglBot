package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore
import org.jetbrains.exposed.sql.transactions.transaction
import org.aeriagloris.persistence.schema.*
import org.jetbrains.exposed.sql.*

import org.joda.time.DateTime

class ListCommand(val store: JdbcStore) : ExtendedCommand("list", "List current lfg/lfm")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        transaction {
            logger.addLogger(StdOutSqlLogger())

            val objs = PlannedActivity.all().filter {
                    it.start > DateTime.now() - 3600 // @todo replace with sql where..
                }.toList().map { act ->
                    "<b>"+act.id+"</b>: <b>"+act.activity.formatName()+"</b>\n" +
                        act.details + "\n" +
                        act.membersFormattedColumn() + "\n" +
                        "<b>" + formatStartTime(act.start) + "</b>\n" +
                        act.joinPrompt() + "\n"
                }.joinToString("\n")

            sendReply(absSender, chat,
                "Planned activities:\n\n"+objs, true)
        }
    }
}

// Event starting in 15 minutes: Iron Banner with dozniak, aero_kamero (4 more can join)
