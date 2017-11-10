package org.aeriagloris.telegram.services

import org.telegram.telegrambots.api.methods.send.SendMessage
import org.telegram.telegrambots.api.objects.Chat
import org.telegram.telegrambots.api.objects.User
import org.telegram.telegrambots.bots.AbsSender
import org.telegram.telegrambots.logging.BotLogger
import org.aeriagloris.persistence.JdbcStore
import org.aeriagloris.persistence.schema.*
import org.jetbrains.exposed.sql.transactions.transaction
import org.joda.time.DateTime
import org.joda.time.DateTimeZone
import org.jetbrains.exposed.sql.*
import org.aeriagloris.rss.RssModule
import org.aeriagloris.rss.model.*
import org.aeriagloris.rss.services.AlertService
import java.text.SimpleDateFormat
import java.util.*
import retrofit2.converter.simplexml.SimpleXmlConverterFactory
import org.aeriagloris.telegram.commands.sendReplyMessage
import mu.KLogging

class AlertsWatcher(val store: JdbcStore) {
    companion object : KLogging() {
        //Mon, 05 Jun 2017 07:41:40 +0000
        val format = SimpleDateFormat("EEE, dd MMM yyyy HH:mm:ss zzz", Locale.ENGLISH)
    }

    fun check(chatId: String, absSender: AbsSender)
    {
        val url = "http://content.ps4.warframe.com/dynamic/"
        val temp = RssModule()
        val client = temp.provideOkHttpClient(temp.provideLoggingInterceptor())
        val alertService = retrofit2.Retrofit.Builder()
            .client(client)
            .addConverterFactory(SimpleXmlConverterFactory.create())
            .baseUrl(url)
            .build()
            .create(AlertService::class.java)

        val response = alertService.getAlerts("rss.php").execute()
        val feed = response.body()
        feed.url = url

        transaction {
            logger.addLogger(Slf4jSqlLogger())

            val items = mutableListOf<Alert>()
            val feedItems = feed.channel?.feedItems ?: emptyList()
            for (feedItem in feedItems) {
                val g = feedItem.guid ?: ""
                val alert = Alert.find { Alerts.guid eq g }.singleOrNull()

                if (alert == null) {
                    items.add(
                        Alert.new {
                            guid = feedItem.guid ?: ""
                            title = feedItem.title ?: ""
                            type = feedItem.type ?: ""
                            startDate = DateTime(format.parse(feedItem.startDate ?: ""))
                            expiryDate = try { DateTime(format.parse(feedItem.expiryDate ?: "")) } catch(x: Exception) { null }
                            faction = feedItem.faction ?: null
                        }
                    )
                }
            }

            // Publish all new alerts (@todo sorted by expiry date)

            for (item in items.filter {i -> i.type == "Alert"}) {
                sendReplyMessage(absSender, chatId.toLong(), "${Emoji.RAISED_FIST} Alert: ${item.title}", true)
            }
        }
    }
}
