package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore
import org.aeriagloris.telegram.services.ActivityIndex
import org.jetbrains.exposed.sql.transactions.transaction
import org.aeriagloris.persistence.schema.*
import org.jetbrains.exposed.sql.*

import org.joda.time.DateTime
import org.joda.time.DateTimeZone
import org.joda.time.format.DateTimeFormat

class LfgCommand(val store: JdbcStore)
    : ExtendedCommand("lfg", "Looking for group (if you look for a fireteam)")
{
    fun usage(absSender: AbsSender, chat: Chat) {
//            "LFG usage: /lfg <b>activity</b> [class] <b>timespec</b>\n"+
        sendReply(absSender, chat,
            "LFG usage: /lfg <b>activity</b> timespec\n"+
            "For a list of activity codes: /lfg activities\n"+
            "Example: /lfg kf tomorrow 23:00\n"+
            "(NB: times are in MSK timezone by default)", true)
    }

    override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
    {
        if (arguments.size < 1) {
            usage(absSender, chat)
            return
        }

        if (arguments[0] == "activities") {
            val builder = StringBuilder("Activities: use a short name:\n")

            for ((k, act) in ActivityIndex.map) {
                builder.append("<b>"+k+"</b>\t" + act.first + " (" + act.second + ")\n")
            }

            sendReply(absSender, chat, builder.toString(), true)
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

                val act1 = ActivityIndex.map[ arguments[0] ]!!
                val act = Activity
                    .find { (Activities.name eq act1.first) and (Activities.mode eq act1.second) }
                    .singleOrNull()

                if (act == null) {
                    sendReply(absSender, chat, "Activity "+arguments[0]+" was not found.")
                } else {
                    val startTime = parseTimeSpec(arguments.drop(1).joinToString(" "))

                    val plannedActivity = PlannedActivity.new {
                        author = dbUser
                        activity = act
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
                        + act.formatName()
                        +" group "+formatStartTime(startTime)+"\n"
                        +plannedActivity.joinPrompt())

                    //sendReply(absSender, "@"+dbUser.telegramName, "Your lfg is added, to set additional details...")
                }
            }
        }
    }
}
