package org.aeriagloris.telegram

import org.telegram.telegrambots.bots.TelegramLongPollingCommandBot
import org.telegram.telegrambots.api.objects.Update
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.telegram.commands.CancelCommand
import org.aeriagloris.telegram.commands.DetailsCommand
import org.aeriagloris.telegram.commands.HelpCommand
import org.aeriagloris.telegram.commands.JoinCommand
import org.aeriagloris.telegram.commands.PsnCommand
import org.aeriagloris.telegram.commands.LfgCommand
import org.aeriagloris.telegram.commands.ListCommand
import org.aeriagloris.telegram.commands.RaidCommand
import org.aeriagloris.telegram.commands.UpdateCommand
import org.aeriagloris.telegram.commands.WhoisCommand
import org.aeriagloris.persistence.JdbcStore
import org.slf4j.LoggerFactory

// https://destinytrialsreport.com/ps/Kayouga
// destinytracker
// guardian.gg

class AeglBot(val store: JdbcStore)
    : TelegramLongPollingCommandBot()
{
    companion object {
        var telegramBotName: String = ""
        var telegramBotToken: String = ""
    }

    val log = LoggerFactory.getLogger(AeglBot::class.java)

    init {
        log.info("Starting bot "+telegramBotName)

        // Telegram Setup
        register(CancelCommand(store))
        register(DetailsCommand(store))
        register(JoinCommand(store))
        register(PsnCommand(store))
        register(LfgCommand(store))
        register(ListCommand(store))
        //register(RaidCommand(store))
        register(UpdateCommand(store))
        register(WhoisCommand(store))
        register(HelpCommand(this))
    }

    override fun getBotToken(): String {
        return telegramBotToken
    }

    override fun getBotUsername(): String {
        return telegramBotName
    }

    override fun processNonCommandUpdate(update: Update) {
        // do nothing for simple chat...
    }

    // @todo Add a thread that would get once a minute a list of planned activities and
    // notify when the time is closing in.
    // e.g.
    // Event starting in 15 minutes: Iron Banner with dozniak, aero_kamero (4 more can join)
}
