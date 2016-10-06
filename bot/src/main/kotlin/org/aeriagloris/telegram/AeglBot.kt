package org.aeriagloris.telegram

import org.telegram.telegrambots.bots.TelegramLongPollingCommandBot
import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.objects.Update
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.telegram.commands.CancelCommand
import org.aeriagloris.telegram.commands.HelpCommand
import org.aeriagloris.telegram.commands.JoinCommand
import org.aeriagloris.telegram.commands.PsnCommand
import org.aeriagloris.telegram.commands.LfgCommand
import org.aeriagloris.telegram.commands.LfmCommand
import org.aeriagloris.telegram.commands.ListCommand
import org.aeriagloris.telegram.commands.RaidCommand
import org.aeriagloris.persistence.JdbcStore
import com.typesafe.config.ConfigFactory

class AeglBot : TelegramLongPollingCommandBot()
{
    val telegramBotToken: String

    init {
        val config = ConfigFactory.load()
        telegramBotToken = config.getString("bot.token")

        // Database Setup
        val jdbcStore = JdbcStore("org.postgresql.Driver", config.getString("bot.database"))

        // Telegram Setup
        register(CancelCommand(jdbcStore))
        register(JoinCommand(jdbcStore))
        register(PsnCommand(jdbcStore))
        register(LfgCommand(jdbcStore))
        register(LfmCommand(jdbcStore))
        register(ListCommand(jdbcStore))
        register(RaidCommand(jdbcStore))
        register(HelpCommand(this))
    }

    override fun getBotToken(): String {
        return telegramBotToken
    }

    override fun getBotUsername(): String {
        return "AeglBot"
    }

    override fun processNonCommandUpdate(update: Update) {
        // do nothing for simple chat...
    }
}
