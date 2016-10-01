package org.aeriagloris.telegram

import org.telegram.telegrambots.TelegramBotsApi

fun main(args: Array<String>) {
    val telegramBotsApi = TelegramBotsApi();
    telegramBotsApi.registerBot(MeinBot());
}
