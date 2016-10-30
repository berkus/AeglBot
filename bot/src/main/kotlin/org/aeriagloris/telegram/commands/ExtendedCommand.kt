package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.bots.commands.BotCommand
import org.telegram.telegrambots.logging.BotLogger
import java.util.concurrent.TimeUnit
import org.joda.time.DateTime
import org.joda.time.format.DateTimeFormat
import com.joestelmach.natty.*
import java.util.TimeZone

abstract class ExtendedCommand(tag: String, text: String) : BotCommand(tag, text)
{
    fun sendReply(absSender: AbsSender, chat: Chat, message: String, isHtml: Boolean = false) {
        val answer = SendMessage()
        answer.setChatId(chat.getId().toString())
        answer.enableHtml(isHtml)
        answer.setText(message)
        answer.disableNotification()
        // @todo make some commands with enabled notifications?
        // maybe make separate notifyMessage() for that

        try {
            absSender.sendMessage(answer)
        } catch (e: TelegramApiException) {
            BotLogger.error("COMMAND", e)
        }
    }

    fun parseTimeSpec(timespec: String): DateTime {
        val parser = Parser(TimeZone.getTimeZone("Europe/Moscow"))
        val groups = parser.parse(timespec)
        return DateTime(groups[0].dates[0])
    }

    fun timeDiffString(duration: Long): String {
        val times = arrayOf(
            TimeUnit.DAYS.toMillis(365),
            TimeUnit.DAYS.toMillis(30),
            TimeUnit.DAYS.toMillis(1),
            TimeUnit.HOURS.toMillis(1),
            TimeUnit.MINUTES.toMillis(1)
        )
        val timesString = arrayOf("year","month","day","hour","minute")

        var dur = Math.abs(duration)
        val res = times.zip(timesString).map { item ->
            val (current, timesStr) = item
            val temp = dur / current
            if (temp > 0) {
                dur -= temp * current
                temp.toString() + " " + timesStr + if (temp != 1L) { "s" } else { "" }
            } else {
                ""
            }
        }.joinToString(" ").trim()

        if ("".equals(res)) {
            return "just now"
        }
        else {
            if (duration > 0) {
                return "in " + res
            }
            else {
                return res + " ago"
            }
        }
    }

    // "Today at 23:00 (starts in 3 hours)"
    fun formatStartTime(time: DateTime): String {
        val prefix = if (time.withTime(0,0,0,0) == DateTime.now().withTime(0,0,0,0)) { 
            "Today"
        } else {
            "on " + DateTimeFormat.forStyle("S-").print(time)
        }

        val prefix2 = " at " + DateTimeFormat.forStyle("-S").print(time)

        val timeDiff = time.getMillis() - DateTime.now().getMillis()
        val infixStr = if (timeDiff <= 0) { " (started " } else { " (starts " }

        return prefix + prefix2 + infixStr + timeDiffString(timeDiff) + ")"
    }
}
