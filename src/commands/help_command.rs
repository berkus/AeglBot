// class HelpCommand(val commandRegistry: ICommandRegistry)
//     : ExtendedCommand("help", "List available commands")
// {
//     override fun execute(absSender: AbsSender, user: User, chat: Chat, arguments: Array<String>)
//     {
//         val helpMessageBuilder = StringBuilder("<b>Help</b> 🚑\n")
//         helpMessageBuilder.append("These are the registered commands for this Bot:\n\n")

//         commandRegistry.getRegisteredCommands().forEach { botCommand: BotCommand ->
//             helpMessageBuilder.append(botCommand.toString()).append("\n\n")
//         }

//         sendReply(absSender, chat, helpMessageBuilder.toString(), true)
//     }
// }
