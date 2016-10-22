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
                    it.start > DateTime.now() - 3600
                }.toList().map { act ->
                "<b>"+act.id+"</b>: "+
                    act.members.toList().joinToString { memb -> memb.user.psnName + " (@" + memb.user.telegramName + ")" }+
                    " going to " + act.activity.name + " " + act.activity.mode +
                    " <b>" + formatStartTime(act.start) + "</b>\n" +
                    "Enter <b>/join "+act.id+"</b> to join this group.\n"
            }.joinToString("\n")

            sendReply(absSender, chat,
                "Planned activities:\n\n"+objs, true)
        }
    }
}

// Event starting in 15 minutes: Iron Banner with dozniak, aero_kamero (4 more can join)
