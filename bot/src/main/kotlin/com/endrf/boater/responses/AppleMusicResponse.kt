package com.endrf.boater.responses

import com.endrf.boater.BotMusicManager
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.util.concurrent.CompletableFuture

@Suppress("PLUGIN_IS_NOT_ENABLED")
@Serializable
data class AppleMusicResponse(
    @SerialName("id") val id: String,
    @SerialName("attributes") private val attributes: Attributes,
    @SerialName("relationships") private val relationships: Relationships? = null
) {
    val title: String get() = attributes.title
    val url: String get() = "https://music.apple.com/us/song/${id}"
    val artist: String get() = attributes.artistList.split(',', '&')[0]
    val artistList: String get() = attributes.artistList
    val artistURL: String get() = ("https://music.apple.com/us/artist/$artist/${relationships?.artists?.data?.get(0)?.id}").replace(' ', '+')
    val artistLogo: () -> String? = {
        val logo = CompletableFuture<String?>()
        val headers = mapOf(
            "Authorization" to "Bearer ${BotMusicManager.appleToken}",
            "Origin" to "https://music.apple.com"
        )
        BotMusicManager.fetchData("https://api.music.apple.com/v1/catalog/us/artists/${relationships?.artists?.data?.get(0)?.id}", "GET", headers, callback = {
            val response = it.body?.get("data")?.jsonArray?.get(0)?.jsonObject?.get("attributes")?.jsonObject?.get("artwork")?.jsonObject?.get("url")?.jsonPrimitive?.content
            logo.complete(response?.replace("{w}x{h}", "80x80"))
        })
        logo.get()
    }
    val cover: String get() = attributes.artwork.url.replace("{w}x{h}", "595x595")
    val durationMs: Int get() = attributes.durationMs
    val releaseDate: String? get() = attributes.releaseDate
    val isrc: String get() = attributes.isrc

    companion object {
        @Serializable
        data class Attributes(@SerialName("name") val title: String,
                              @SerialName("artistName") val artistList: String,
                              @SerialName("durationInMillis") val durationMs: Int,
                              @SerialName("artwork") val artwork: Artwork,
                              @SerialName("releaseDate") val releaseDate: String? = null,
                              @SerialName("isrc") val isrc: String
        ) {
            @Serializable
            data class Artwork(@SerialName("url") val url: String)
        }

        @Serializable
        data class Relationships(@SerialName("artists") val artists: Artists) {

            @Serializable
            data class Artists(@SerialName("data") val data: List<Data>) {

                @Serializable
                data class Data(@SerialName("id") val id: String)
            }
        }
    }
}