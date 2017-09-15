package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.replykeyboard.buttons.KeyboardRow
import org.telegram.telegrambots.api.objects.replykeyboard.ReplyKeyboardMarkup
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore
import org.jetbrains.exposed.sql.transactions.transaction
import org.aeriagloris.persistence.schema.*
import org.jetbrains.exposed.sql.*

import org.joda.time.DateTime
import org.joda.time.DateTimeZone
import org.joda.time.format.DateTimeFormat

class LfgCommand(val store: JdbcStore)
    : ExtendedCommand("lfg", "Looking for group (if you want to create an event)")
{
    fun usage(absSender: AbsSender, chat: Chat) {
//            "LFG usage: /lfg <b>activity</b> [class] <b>timespec</b>\n"+
        sendReply(absSender, chat,
            "LFG usage: /lfg <b>activity</b> timespec\n"+
            "For a list of activity codes: /lfg activities\n"+
            "Example: /lfg kf tomorrow 23:00\n"+
            "(NB: times are in MSK timezone by default)", true)
    }

    fun interactive(absSender: AbsSender, chat: Chat) {
        val message = SendMessage()
        message.setText("Pick a game from the list")

        val row1 = KeyboardRow()
        // Set each button, you can also use KeyboardButton objects if you need something else than text
        row1.add("Destiny")
        row1.add("Destiny 2")
        row1.add("TESO")

        val row2 = KeyboardRow()
        row2.add("Alienation")
        row2.add("Titanfall 2")
        row2.add("Cancel")

        val keyboardMarkup = ReplyKeyboardMarkup()
        keyboardMarkup.setKeyboard(listOf(row1, row2))

        message.setReplyMarkup(keyboardMarkup)
        sendMessage(absSender, chat, message)
    }

    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        if (arguments.size < 1) {
            interactive(absSender, chat)
            //usage(absSender, chat)
            return
        }

        if (arguments[0] == "activities") {
            transaction {
                var text = "Activities: use a short name:\n";
                val games = ActivityShortcuts.slice(ActivityShortcuts.game).selectAll().withDistinct().toList()
                    .map { game -> game[ActivityShortcuts.game] }.sorted()

                for (game in games) {
                    text += "*** <b>"+game+"</b>:\n" +
                        ActivityShortcut.find { ActivityShortcuts.game eq game }.toList().sortedBy { ActivityShortcuts.name }.map { act ->
                            "<b>"+act.name+"</b>\t"+act.link.formatName()
                        }.joinToString("\n") + "\n"
                }

                sendReply(absSender, chat, text, true)
            }
            return
        }

        if (arguments.size < 2) {
            usage(absSender, chat)
            return
        }

        transaction {
            logger.addLogger(StdOutSqlLogger())

            val dbUser = Guardian.find { Guardians.telegramName eq user.getUserName() }.singleOrNull()

            if (dbUser == null) {
                sendReply(absSender, chat, "You need to link your PSN account first: use /psn command")
            } else {

                val act = ActivityShortcut
                    .find { ActivityShortcuts.name eq arguments[0] }
                    .singleOrNull()

                if (act == null) {
                    sendReply(absSender, chat, "Activity "+arguments[0]+" was not found.")
                } else {
                    val startTime = parseTimeSpec(arguments.drop(1).joinToString(" "))

                    val plannedActivity = PlannedActivity.new {
                        author = dbUser
                        activity = act.link
                        start = startTime
                        // set these using "/details id text" command
                        details = ""
                    }

                    PlannedActivityMember.new {
                        this.user = dbUser
                        this.activity = plannedActivity
                    }

                    sendReply(absSender, chat, // Todo: always post to lfg chat?
                        dbUser.formatName() + " is looking for "
                        + act.link.formatName()
                        +" group "+formatStartTime(startTime)+"\n"
                        +plannedActivity.joinPrompt())

                    //sendReply(absSender, "@"+dbUser.telegramName, "Your lfg is added, to set additional details...")
                }
            }
        }
    }
}
