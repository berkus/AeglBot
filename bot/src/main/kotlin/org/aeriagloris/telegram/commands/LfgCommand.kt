package org.aeriagloris.telegram.commands

import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore
import org.aeriagloris.telegram.services.ActivityIndex
import org.jetbrains.exposed.sql.transactions.transaction
import org.aeriagloris.persistence.schema.*

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

///            val user = User.find { Users.telegramName eq user.getUserName() }.single()
//            val user = Users.find { Users.name eq user.getUserName() }
//
//            val userId = store.lookupTelegramUserName()
//
//            if (userId == null) {
//                sendReply(absSender, chat, "You need to link your PSN account first: use /psn command")
//                return
//            }
//
//            val lookedUpActivityId = store.lookupActivity(arguments[0])
//
//            if (lookedUpActivityId == null) {
//                sendReply(absSender, chat, "Activity "+arguments[0]+" was not found.")
//                return
//            }
//
//            var time: String
//            var tz: DateTimeZone
//
//            if (arguments[1].count() > 11) {
//                time = arguments[1].substring(0, 11)
//                tz = DateTimeZone.forID(arguments[1].substring(11))
//            } else {
//                time = arguments[1]
//                tz = DateTimeZone.UTC
//            }
//
//            val fmt = DateTimeFormat.forPattern("yyyy.MM.dd-HH:mm").withZone(tz);
//            val start = fmt.parseDateTime("2016."+time)
//            //sendReply(absSender, chat, "Defined moment to be "+start)
//
//            val activity = PlannedActivityData(
//                authorId = userId,
//                activityId = lookedUpActivityId,
//                start = start,
//                details = "")
//
//            val actId = store.createPlannedActivity(activity)
//
//            if (actId == null) {
//                sendReply(absSender, chat, "Internal error: couldn't create activity")
//                return
//            }
//
//            // Add self as activity member
//            val member = PlannedActivityMember()
//
//
//            val userData = store.loadUserData(userId)
//
//            if (userData == null) {
//                sendReply(absSender, chat, "Internal error: couldn't load user information")
//                return
//            }
//
//            val activityData = store.loadActivityData(actId)
//
//            if (activityData == null) {
//                sendReply(absSender, chat, "Internal error: couldn't load activity information")
//                return
//            }

        }

//"Today at 23:00 MSK (starts in 3 hours)" = formatStartTime(time)

//        sendReply(absSender, chat,
//            userData.psnName + " (@" + userData.telegramName + ") is looking for "
//            + activityData.name + " " + activityData.mode
//            +" group "+formatStartTime(start)+"\n"
//            + "Enter /join "+actId+" to join this group.")
    }
}

// Event starting in 15 minutes: Iron Banner with dozniak, aero_kamero (4 more can join)

