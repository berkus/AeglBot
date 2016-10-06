package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore

class LfgCommand(val store: JdbcStore) : ExtendedCommand("lfg", "Looking for group (if you look for a fireteam)")
{
    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        
        // [0] activity code
        // [1] when (timezone?)
        // [2] (optional) how many more guardians needed
        // /lfg kf 4
        // /lfg ib
        sendReply(absSender, chat, "Looking for group")
    }
}


// activity map
// kf[n|h] = King's Fall
// cr[n|h] = Crota's End
// vog[n|h] = Vault of Glass
// pvp = Crucible any
// pve = 
// ib = iron banner
// too = 
