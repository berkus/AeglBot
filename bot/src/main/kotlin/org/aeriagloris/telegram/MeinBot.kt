package org.aeriagloris.telegram

import org.telegram.telegrambots.bots.TelegramLongPollingBot
import org.telegram.telegrambots.TelegramApiException
import org.telegram.telegrambots.api.objects.Update
import org.telegram.telegrambots.api.methods.send.SendMessage

class MeinBot : TelegramLongPollingBot()
{
    override fun getBotToken(): String {
        return "" // FIXME
    }

    override fun getBotUsername(): String {
        return "AeglBot"
    }

    override fun onUpdateReceived(update: Update) {
        if (update.hasMessage()) {
            val message = update.getMessage()

            //check if the message has text. it could also contain for example a location ( message.hasLocation() )
            if (message.hasText()) {
                //create an object that contains the information to send back the message
                val sendMessageRequest = SendMessage()
                sendMessageRequest.setChatId(message.getChatId().toString()) //who should get from the message the sender that sent it.
                sendMessageRequest.setText("you said: " + message.getText())
                try {
                    sendMessage(sendMessageRequest) //at the end, so some magic and send the message ;)
                } catch (e: TelegramApiException) {
                    //do some error handling
                }
            }
        }
    }
}
