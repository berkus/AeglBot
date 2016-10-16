package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User as TelegramUser
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore
import org.jetbrains.exposed.sql.transactions.transaction
import org.aeriagloris.persistence.schema.*
import org.jetbrains.exposed.sql.*

class ListCommand(val store: JdbcStore) : ExtendedCommand("list", "List current lfg/lfm")
{
    override fun execute(absSender: AbsSender, user: TelegramUser, chat: Chat, arguments: Array<String>)
    {
        transaction {
            val activs = PlannedActivity.all()
            //.map {
            //    "<b>"+it.id+"</b>: "+it.members.joinToString { it.user.psnName + " (@" + it.user.telegramName + ")" }+
            //    " going to " + it.activity.name + " " + it.activity.mode +
            //    " at <b>" + formatStartTime(it.start) + "</b>"
            //}

            sendReply(absSender, chat,
                "Planned activities:\n"+activs
                + "Enter /join <b>id</b> to join group.", true)

        }

// Sample output: (same output for List command)

        // dozniak (@berkus) is looking for Iron Banner group Today at 23:00 MSK (starts in 3 hours)
        // Enter /join 3 to join this group.

    }
}
