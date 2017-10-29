package org.aeriagloris.rss

import dagger.Module
import dagger.Provides
import okhttp3.Cache
import okhttp3.CacheControl
import okhttp3.Interceptor
import okhttp3.OkHttpClient
import okhttp3.logging.HttpLoggingInterceptor
import java.io.File
import java.util.concurrent.TimeUnit
import org.jetbrains.anko.*

@Module
class RssModule() : AnkoLogger {

    val cacheSize: Long = 10 * 1024 * 1024
    val cacheTimeSec = 10

    val cacheInterceptor: Interceptor
        get() = Interceptor {
            val response = it.proceed(it.request())
            val cacheControl = CacheControl.Builder()
                    .maxAge(cacheTimeSec, TimeUnit.SECONDS)
                    .build()

            response.newBuilder()
                    .header("Cache-Control", cacheControl.toString())
                    .build()
        }

    @Provides
    fun provideLoggingInterceptor(): HttpLoggingInterceptor {
        val interceptor = HttpLoggingInterceptor(HttpLoggingInterceptor.Logger { message -> debug(message) })
        //interceptor.level = if (BuildConfig.DEBUG) HttpLoggingInterceptor.Level.BODY else HttpLoggingInterceptor.Level.NONE
        return interceptor
    }

    @Provides
    fun provideOkHttpClient(
            //context: Context,
            loggingInterceptor: HttpLoggingInterceptor
    ): OkHttpClient {
        //val cache = Cache(File(context.cacheDir, "http-cache"), cacheSize)
        return OkHttpClient.Builder()
                .addInterceptor(loggingInterceptor)
                .addInterceptor(cacheInterceptor)
                //.addNetworkInterceptor(StethoInterceptor())
                //.cache(cache)
                .build()
    }
}
