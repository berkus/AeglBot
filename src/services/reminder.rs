// class Reminder(val store: JdbcStore)
// {
//     // Check for upcoming events and remind to specified LFG chat
//     fun check(chatId: String)
//     {
//         transaction {
//             logger.addLogger(Slf4jSqlLogger())

//             val minutesAgo = DateTime.now(DateTimeZone.forID("Europe/Moscow")).minusMinutes(15)
//             val objs = PlannedActivity.find {
//                     PlannedActivities.start greaterEq minutesAgo
//                 }.toList().sortedBy { it.start }.map { act ->
//                     "<b>"+act.id+"</b>: <b>"+act.activity.formatName()+"</b>\n" +
//                         act.detailsFormatted() +
//                         act.membersFormattedColumn() + "\n" +
//                         //"<b>" + formatStartTime(act.start) + "</b>\n" +
//                         act.joinPrompt() + "\n"
//                 }.joinToString("\n")

//             if ("".equals(objs)) {
//                 //sendReply(absSender, chat, "No activities planned, add something with /lfg")
//             } else {
//                 print(objs)
//                 //sendReply(absSender, chat, "Planned activities:\n\n" + objs, true)
//             }
//         }
//     }
// }
