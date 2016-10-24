package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore

class DetailsCommand(val store: JdbcStore) : ExtendedCommand("details", "Set group details")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        sendReply(absSender, chat, "You are not going - please provide activity id in arguments")
        if (arguments.size < 2) {
            sendReply(absSender, chat, "To join a fireteam provide fireteam id\n"
            + "Fireteam IDs are available from output of /list command.")
            return
        }
/*
        transaction {
            logger.addLogger(StdOutSqlLogger())

            val dbUser = Guardian.find { Guardians.telegramName eq user.getUserName() }.singleOrNull()

            if (dbUser == null) {
                sendReply(absSender, chat, "You need to link your PSN account first: use /psn command")
            } else {

                val planned = PlannedActivity
                    .findById(arguments[0].toInt())

                if (planned == null) {
                    sendReply(absSender, chat, "Activity "+arguments[0]+" was not found.")
                } else {

                    PlannedActivityMember.new {
                        this.user = dbUser
                        this.activity = planned
                    }

                    sendReply(absSender, chat,
                        dbUser.psnName + " (@" + dbUser.telegramName + ") is joining "
                        + planned.activity.name + " " + planned.activity.mode
                        +" group\n"
                        +planned.members.toList().joinToString { it.user.psnName }+" are going\n"
                        + "Enter /join "+planned.id+" to join this group.")
                }
            }
        }*/
    }
}
