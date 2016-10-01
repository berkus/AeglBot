package org.aeriagloris.telegram

import org.telegram.telegrambots.TelegramBotsApi
import org.wasabifx.wasabi.app.AppServer

fun main(args: Array<String>) {
    val telegramBotsApi = TelegramBotsApi();
    telegramBotsApi.registerBot(MeinBot());

    val server = AppServer()
    server.get("/", { response.send("Hello, go away!") })
    server.start()
}
