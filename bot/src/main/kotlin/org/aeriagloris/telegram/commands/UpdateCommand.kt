package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.aeriagloris.persistence.JdbcStore
import org.jetbrains.exposed.sql.transactions.transaction
import org.aeriagloris.persistence.schema.Activity

class UpdateCommand(val store: JdbcStore) : ExtendedCommand("update", "Update activity database")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        transaction {
            Activity.new {
                name = "Crucible"; mode = "Private Tournament"
                minFireteamSize = 1; maxFireteamSize = 12
            }
            Activity.new {
                name = "Vanguard"; mode = "Challenge of Elders"
                minFireteamSize = 1; maxFireteamSize = 3
                minLight = 320
            }
            Activity.new {
                name = "Vanguard"; mode = "Prison of Elders"
                minFireteamSize = 1; maxFireteamSize = 3
            }
        }

        sendReply(absSender, chat, "Database updated.")
    }
}
