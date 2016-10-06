package org.aeriagloris.telegram

import org.telegram.telegrambots.bots.TelegramLongPollingCommandBot
import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.objects.Update
import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.telegram.services.Emoji
import org.aeriagloris.telegram.commands.CancelCommand
import org.aeriagloris.telegram.commands.HelpCommand
import org.aeriagloris.telegram.commands.JoinCommand
import org.aeriagloris.telegram.commands.PsnCommand
import org.aeriagloris.telegram.commands.LfgCommand
import org.aeriagloris.telegram.commands.LfmCommand
import org.aeriagloris.telegram.commands.ListCommand
import org.aeriagloris.telegram.commands.RaidCommand
import org.aeriagloris.persistence.JdbcStore
import com.typesafe.config.ConfigFactory

class AeglBot : TelegramLongPollingCommandBot()
{
    val telegramBotToken: String

    init {
        val config = ConfigFactory.load()
        telegramBotToken = config.getString("bot.token")

        // Database Setup
        val jdbcStore = JdbcStore("org.postgresql.Driver", config.getString("bot.database"))

        // Telegram Setup
        register(CancelCommand(jdbcStore))
        register(JoinCommand(jdbcStore))
        register(PsnCommand(jdbcStore))
        register(LfgCommand(jdbcStore))
        register(LfmCommand(jdbcStore))
        register(ListCommand(jdbcStore))
        register(RaidCommand(jdbcStore))
        val helpCommand = HelpCommand(this)

        registerDefaultAction { absSender, message ->
            val commandUnknownMessage = SendMessage()
            commandUnknownMessage.setChatId(message.getChatId().toString())
            commandUnknownMessage.setText("The command '" + message.getText() + "' is not known by this bot. Here comes some help " + Emoji.AMBULANCE)
            try {
                absSender.sendMessage(commandUnknownMessage)
            } catch (e: TelegramApiException) {
                BotLogger.error("MEINBOT", e)
            }
            helpCommand.execute(absSender, message.getFrom(), message.getChat(), arrayOf<String>())
        }
    }

    override fun getBotToken(): String {
        return telegramBotToken
    }

    override fun getBotUsername(): String {
        return "AeglBot"
    }

    override fun processNonCommandUpdate(update: Update) {
        if (update.hasMessage()) {
            val message = update.getMessage()

            //check if the message has text. it could also contain for example a location ( message.hasLocation() )
            if (message.hasText()) {
                //create an object that contains the information to send back the message
                val sendMessageRequest = SendMessage()
                sendMessageRequest.setChatId(message.getChatId().toString()) //who should get from the message the sender that sent it.
                sendMessageRequest.setText("you said: " + message.getText())
                try {
                    sendMessage(sendMessageRequest)
                } catch (e: TelegramApiException) {
                    //do some error handling
                }
            }
        }
    }
}
