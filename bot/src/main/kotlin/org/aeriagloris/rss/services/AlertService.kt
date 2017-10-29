package org.aeriagloris.rss.services

import org.aeriagloris.rss.model.Feed
import retrofit2.Call
import retrofit2.http.GET
import retrofit2.http.Path

interface AlertService {
    @GET("{feed}")
    fun getAlerts(@Path("feed") feed: String): Call<Feed>
}
