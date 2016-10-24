package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.aeriagloris.persistence.JdbcStore
import org.jetbrains.exposed.sql.transactions.transaction
import org.aeriagloris.persistence.schema.Activity

class UpdateCommand(val store: JdbcStore) : ExtendedCommand("update", "Update activity database (for admin)")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        transaction {
        }

        sendReply(absSender, chat, "Database already up to date.")
    }
}
