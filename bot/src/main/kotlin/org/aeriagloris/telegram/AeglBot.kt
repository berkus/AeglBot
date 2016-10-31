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
import org.aeriagloris.telegram.commands.LfmCommand
import org.aeriagloris.telegram.commands.ListCommand
import org.aeriagloris.telegram.commands.RaidCommand
import org.aeriagloris.telegram.commands.UpdateCommand
import org.aeriagloris.telegram.commands.WhoisCommand
import org.aeriagloris.persistence.JdbcStore
import com.typesafe.config.ConfigFactory
import java.util.TimeZone

// https://destinytrialsreport.com/ps/Kayouga
// destinytracker
// guardian.gg

class AeglBot : TelegramLongPollingCommandBot()
{
    val telegramBotToken: String
    val telegramBotName: String

    init {
        val config = ConfigFactory.load()
        telegramBotToken = config.getString("bot.token")
        telegramBotName = config.getString("bot.name")

        TimeZone.setDefault(TimeZone.getTimeZone(config.getString("bot.timezone")))

        // Database Setup
        val jdbcStore = JdbcStore(config.getString("bot.driver"), config.getString("bot.database"))

        // Telegram Setup
        register(CancelCommand(jdbcStore))
        register(DetailsCommand(jdbcStore))
        register(JoinCommand(jdbcStore))
        register(PsnCommand(jdbcStore))
        register(LfgCommand(jdbcStore))
        //register(LfmCommand(jdbcStore))
        register(ListCommand(jdbcStore))
        //register(RaidCommand(jdbcStore))
        register(UpdateCommand(jdbcStore))
        register(WhoisCommand(jdbcStore))
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
