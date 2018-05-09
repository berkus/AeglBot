// class ActivitiesCommand(val store: JdbcStore)
//     : ExtendedCommand("activities", "List available activity shortcuts")
// {
//     override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
//     {
//         transaction {
//             var text = "Activities: use a short name:\n";
//             val games = ActivityShortcuts.slice(ActivityShortcuts.game).selectAll().withDistinct().toList()
//                 .map { game -> game[ActivityShortcuts.game] }.sorted()

//             for (game in games) {
//                 text += "*** <b>${game}</b>:\n" +
//                     ActivityShortcut.find { ActivityShortcuts.game eq game }.toList().sortedBy { ActivityShortcuts.name }.map { act ->
//                         "<b>${act.name}</b>\t${act.link.formatName()}"
//                     }.joinToString("\n") + "\n"
//             }

//             sendReply(absSender, chat, text, true)
//         }
//     }
// }
