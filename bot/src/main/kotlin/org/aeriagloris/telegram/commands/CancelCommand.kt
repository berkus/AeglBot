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

class CancelCommand(val store: JdbcStore) : ExtendedCommand("cancel", "Cancel joining fireteam")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        if (arguments.size != 1) {
            sendReply(absSender, chat, "To leave a fireteam provide fireteam id\n"
            + "Fireteam IDs are available from output of /list command.")
            return
        }

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
                    val member = PlannedActivityMember.find {
                        (PlannedActivityMembers.userId eq dbUser.id) and
                        (PlannedActivityMembers.plannedActivityId eq planned.id)
                    }.singleOrNull()

                    if (member == null) {
                        sendReply(absSender, chat, "You are not part of this group.")
                    } else {
                        member.delete()

                        var suffix = planned.membersFormatted() +" are going\n"+
                                     "Enter "+planned.joinLink()+" to join this group."

                        if (planned.members.count() == 0) {
                            planned.delete()
                            suffix = "This fireteam is disbanded and can no longer be joined."
                        }

                        sendReply(absSender, chat,
                            dbUser.formatName() + " has left " + planned.activity.formatName() + " group\n"
                            +suffix)
                    }
                }
            }
        }
    }
}
