package com.endrf.boater.responses

import com.endrf.boater.BotMusicManager
import com.endrf.boater.dotenv
import io.ktor.util.reflect.*
import kotlin.time.Duration
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.*
import org.dotenv.vault.dotenvVault
import java.util.concurrent.CompletableFuture

@Suppress("PLUGIN_IS_NOT_ENABLED")
@Serializable
data class YoutubeMusicResponse(
    @SerialName("id") private val idElement: JsonElement,
    @SerialName("snippet") private val snippet: Snippet,
    @SerialName("contentDetails") private val contentDetails: ContentDetails? = null
) {
    val id: String get() = if (idElement.instanceOf(JsonObject::class)) idElement.jsonObject["videoId"]?.jsonPrimitive?.content!! else idElement.jsonPrimitive.content
    val title: String get() = snippet.title.replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
    val artist: String get() = snippet.channelTitle
    val artistLogo: () -> String? = {
        val logo = CompletableFuture<String?>()
        BotMusicManager.fetchData("https://www.googleapis.com/youtube/v3/channels?id=${snippet.channelId}&part=snippet&key=${dotenv["YOUTUBE_API_KEY"]}", "GET") {
            val icon = it.body?.get("items")?.jsonArray?.get(0)?.jsonObject?.get("snippet")?.jsonObject?.get("thumbnails")?.jsonObject?.get("default")?.jsonObject?.get("url")?.jsonPrimitive?.content
            if (icon == null) logo.complete(null)
            else logo.complete(icon)
        }
        logo.get()
    }
    val artistURL: () -> String? = {
        val logo = CompletableFuture<String?>()
        BotMusicManager.fetchData("https://www.googleapis.com/youtube/v3/channels?id=${snippet.channelId}&part=snippet&key=${dotenv["YOUTUBE_API_KEY"]}", "GET") {
            val customUrl = it.body?.get("items")?.jsonArray?.get(0)?.jsonObject?.get("snippet")?.jsonObject?.get("customUrl")?.jsonPrimitive?.content
            if (customUrl == null) logo.complete(null)
            else logo.complete("https://www.youtube.com/$customUrl")
        }
        logo.get()
    }
    val url: String get() = "https://www.youtube.com/watch?v=$id"
    val cover: String get() = if (snippet.thumbnails.maxres == null) snippet.thumbnails.high.url else snippet.thumbnails.maxres.url
    val durationMs: Int get() = contentDetails?.duration?.let { Duration.parseIsoString(it).inWholeMilliseconds.toInt() } ?: -1
    val releaseDate: String get() = snippet.publishedAt.split('T').first()

    companion object {
        @Serializable
        data class Snippet(
            @SerialName("title") val title: String,
            @SerialName("channelTitle") val channelTitle: String,
            @SerialName("channelId") val channelId: String,
            @SerialName("thumbnails") val thumbnails: Thumbnails,
            @SerialName("publishedAt") val publishedAt: String
        ) {

            @Serializable
            data class Thumbnails(@SerialName("default") val high: High, @SerialName("maxres") val maxres: MaxRes? = null) {

                @Serializable
                data class High(@SerialName("url") val url: String)

                @Serializable
                data class MaxRes(@SerialName("url") val url: String)
            }
        }

        @Serializable
        data class ContentDetails(
            @SerialName("duration") val duration: String?
        )
    }
}
