package org.aeriagloris.telegram

import org.telegram.telegrambots.ApiContextInitializer
import org.telegram.telegrambots.TelegramBotsApi
import org.aeriagloris.persistence.JdbcStore
import com.typesafe.config.ConfigFactory
import java.util.TimeZone

fun main(args: Array<String>) {
    ApiContextInitializer.init()
    val telegramBotsApi = TelegramBotsApi()

    val config = ConfigFactory.load()
    val botToken = config.getString("bot.token")
    val botName = config.getString("bot.name")
    val lfgChat = config.getString("bot.lfgChatId")

    TimeZone.setDefault(TimeZone.getTimeZone(config.getString("bot.timezone")))

    val jdbcStore = JdbcStore(config.getString("bot.driver"), config.getString("bot.database"))

    // Because of idiotic TelegramBots API we could not set bot name from config easily
    // (it must be available in member function BEFORE we construct an instance of AeglBot)
    AeglBot.telegramBotToken = botToken

    telegramBotsApi.registerBot(AeglBot(botName, jdbcStore, lfgChat))
}
