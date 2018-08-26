fun sendReplyMessage(absSender: AbsSender, chatId: Long, message: String, isHtml: Boolean = false) {
    val answer = SendMessage()
    answer.setChatId(chatId)
    answer.enableHtml(isHtml)
    answer.setText(message)
    answer.disableNotification()
    answer.disableWebPagePreview()
    // @todo make some commands with enabled notifications?
    // maybe make separate notifyMessage() for that

    try {
        absSender.sendMessage(answer)
    } catch (e: TelegramApiException) {
        BotLogger.error("COMMAND", e)
    }
}

fun sendMessage(absSender: AbsSender, chat: Chat, message: SendMessage) {
    message.setChatId(chat.id)
    message.disableNotification()
    message.disableWebPagePreview()

    try {
        absSender.sendMessage(message)
    } catch (e: TelegramApiException) {
        BotLogger.error("COMMAND", e)
    }
}

fun sendReply(absSender: AbsSender, chat: Chat, message: String, isHtml: Boolean = false) {
    sendReplyMessage(absSender, chat.id, message, isHtml)
}

fun parseTimeSpec(timespec: String): DateTime {
    val parser = Parser(TimeZone.getTimeZone("Europe/Moscow")) // @todo bot.timezone
    val groups = parser.parse(timespec)
    return DateTime(groups[0].dates[0])
}
