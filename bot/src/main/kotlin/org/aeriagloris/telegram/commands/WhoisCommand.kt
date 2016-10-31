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

class WhoisCommand(val store: JdbcStore) : ExtendedCommand("whois", "Query telegram or PSN id")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        if (arguments.size != 1) {
            sendReply(absSender, chat, "To query user provide his @TelegramId (starting with @) or PsnId")
            return
        }

        transaction {
            logger.addLogger(StdOutSqlLogger())

            val dbUser = Guardian.find { Guardians.telegramName eq user.getUserName() }.singleOrNull()

            if (dbUser == null) {
                sendReply(absSender, chat, "You need to link your PSN account first: use /psn command")
            } else {
                val name = arguments[0]

                val guardian = if (name.startsWith("@")) {
                        Guardian.find { Guardians.telegramName eq name.drop(1) }.singleOrNull()
                    } else {
                        Guardian.find { Guardians.psnName eq name }.singleOrNull()
                    }

                if (guardian == null) {
                    sendReply(absSender, chat, "Guardian "+name+" was not found.")
                } else {
                    sendReply(absSender, chat, "Guardian @"+guardian.telegramName+" PSN "+guardian.psnName)
                }
            }
        }
    }
}
