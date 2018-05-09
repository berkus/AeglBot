// class ListCommand(val store: JdbcStore) : ExtendedCommand("list", "List current events")
// {
//     override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
//     {
//         transaction {
//             val hourAgo = DateTime.now(DateTimeZone.forID("Europe/Moscow")).minusHours(1)
//             val objs = PlannedActivity.find {
//                     PlannedActivities.start greaterEq hourAgo
//                 }.toList().sortedBy { it.start }.map { act ->
//                     "<b>${act.id}</b>: <b>${act.activity.formatName()}</b>\n" +
//                         act.detailsFormatted() +
//                         act.membersFormattedColumn() + "\n" +
//                         "⏰ <b>${formatStartTime(act.start)}</b>\n" +
//                         act.joinPrompt() + "\n"
//                 }.joinToString("\n")

//             if ("".equals(objs)) {
//                 sendReply(absSender, chat, "No activities planned, add something with /lfg")
//             } else {
//                 sendReply(absSender, chat, "Planned activities:\n\n" + objs, true)
//             }
//         }
//     }
// }
