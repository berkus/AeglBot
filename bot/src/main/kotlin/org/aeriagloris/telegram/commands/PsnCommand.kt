package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger

import org.aeriagloris.persistence.JdbcStore
import org.aeriagloris.persistence.schema.*
import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.dao.*
import org.jetbrains.exposed.sql.transactions.transaction

class PsnCommand(val store: JdbcStore) : ExtendedCommand("psn", "Link your telegram user to PSN")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        if (arguments.size != 1) {
            sendReply(absSender, chat, "Usage: /psn <b>psnid</b>\n"
                +"For example: /psn KPOTA_B_ATEOHE", true)
            return
        }

        transaction {
            logger.addLogger(Slf4jSqlLogger())

            val dbUser = Guardian.find { Guardians.telegramName eq user.getUserName() }.singleOrNull()
            if (dbUser == null) {
                Guardian.new {
                    telegramName = user.getUserName()
                    telegramId = user.getId()
                    psnName = arguments[0]
                }
                sendReply(absSender, chat, "Linking telegram @"+user.getUserName()+" with psn "+arguments[0])
            } else {
                sendReply(absSender, chat, "Your telegram @"+user.getUserName()+" is already linked with psn "+dbUser.psnName)
            }
        }

    }
}
