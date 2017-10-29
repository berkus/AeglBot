package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore
import org.jetbrains.exposed.sql.transactions.transaction
import org.aeriagloris.persistence.schema.*
import org.jetbrains.exposed.sql.*
import org.aeriagloris.telegram.services.Emoji

import org.joda.time.DateTime
import org.joda.time.DateTimeZone

class ListCommand(val store: JdbcStore) : ExtendedCommand("list", "List current events")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        transaction {
            val hourAgo = DateTime.now(DateTimeZone.forID("Europe/Moscow")).minusHours(1)
            val objs = PlannedActivity.find {
                    PlannedActivities.start greaterEq hourAgo
                }.toList().sortedBy { it.start }.map { act ->
                    "<b>${act.id}</b>: <b>${act.activity.formatName()}</b>\n" +
                        act.detailsFormatted() +
                        act.membersFormattedColumn() + "\n" +
                        "${Emoji.ALARM_CLOCK} <b>${formatStartTime(act.start)}</b>\n" +
                        act.joinPrompt() + "\n"
                }.joinToString("\n")

            if ("".equals(objs)) {
                sendReply(absSender, chat, "No activities planned, add something with /lfg")
            } else {
                sendReply(absSender, chat, "Planned activities:\n\n" + objs, true)
            }
        }
    }
}
