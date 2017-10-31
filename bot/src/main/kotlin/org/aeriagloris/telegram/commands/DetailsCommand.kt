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

class DetailsCommand(val store: JdbcStore) : ExtendedCommand("details", "Set group details as freeform text")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        if (arguments.size < 2) {
            sendReply(absSender, chat, "To update fireteam details enter /details ID freeform text\n"
            + "To delete details use /details ID del.\n"
            + "Fireteam IDs are available from output of /list command.")
            return
        }

        transaction {
            logger.addLogger(Slf4jSqlLogger())

            val dbUser = Guardian.find { Guardians.telegramName eq user.getUserName() }.singleOrNull()

            if (dbUser == null) {
                sendReply(absSender, chat, "You need to link your PSN account first: use /psn command")
            } else {

                val planned = PlannedActivity
                    .findById(arguments[0].toInt())

                if (planned == null) {
                    sendReply(absSender, chat, "Activity ${arguments[0]} was not found.")
                } else {

                    planned.details = if (arguments[1] == "del") { "" }
                                      else { arguments.drop(1).joinToString(" ") }

                    sendReply(absSender, chat, "Details updated.")
                }
            }
        }
    }
}
