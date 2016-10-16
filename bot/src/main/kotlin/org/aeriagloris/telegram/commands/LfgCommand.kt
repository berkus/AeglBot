package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User as TelegramUser
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
import java.util.TimeZone

class LfgCommand(val store: JdbcStore)
    : ExtendedCommand("lfg", "Looking for group (if you look for a fireteam)")
{
    fun usage(absSender: AbsSender, chat: Chat) {
        sendReply(absSender, chat,
            "LFG usage: /lfg <b>activity</b> MM.DD-HH:MM <b>[additional description]</b>\n"+
            "For a list of activity codes: /lfg activities\n"+
            "Example: /lfg kf 10.10-23:00\n"+
            "(NB: times are in MSK timezone)", true)
    }

    override fun execute(absSender: AbsSender, user: TelegramUser, chat: Chat, arguments: Array<String>)
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
            val dbUser = User.find { Users.telegramName eq user.getUserName() }.singleOrNull()

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
                    val fmt = if (arguments[1].count() > 11) {
                            DateTimeFormat.forPattern("yyyy.MM.dd-HH:mmzzz")
                        } else {
                            DateTimeFormat.forPattern("yyyy.MM.dd-HH:mm").withZone(DateTimeZone.UTC)
                        }

                    val startTime = fmt.parseDateTime("2016."+arguments[1])
                    sendReply(absSender, chat, "Defined moment to be "+startTime)
        
                    val plannedActivity = PlannedActivity.new {
                        author = dbUser
                        activity = act
                        start = startTime
                        details = arguments.drop(2).joinToString(" ")
                    }

                    PlannedActivityMember.new {
                        this.user = dbUser
                        this.activity = plannedActivity
                    }

                    sendReply(absSender, chat,
                        dbUser.psnName + " (@" + dbUser.telegramName + ") is looking for "
                        + act.name + " " + act.mode
                        +" group "+formatStartTime(startTime)+"\n"
                        + "Enter /join "+plannedActivity.id+" to join this group.")
                }
            }
//"Today at 23:00 MSK (starts in 3 hours)" = formatStartTime(time)
        }
    }
}

// Event starting in 15 minutes: Iron Banner with dozniak, aero_kamero (4 more can join)

