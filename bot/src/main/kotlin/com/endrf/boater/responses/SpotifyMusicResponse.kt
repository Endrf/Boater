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
data class SpotifyMusicResponse(
    @SerialName("id") val id: String,
    @SerialName("name") val title: String,
    @SerialName("duration_ms") val durationMs: Int,
    @SerialName("artists") private val artists: List<Artists>?,
    @SerialName("album") private val album: Album,
    @SerialName("external_urls") private val external_urls: ExternalURLs,
    @SerialName("external_ids") private val external_ids: ExternalIDs
) {
    val url: String get() = external_urls.spotify
    val artist: String get() = artists?.get(0)?.name!!
    val artistList: String get() {
        val nameArr = ArrayList<String>()
        artists?.forEach { nameArr.add(it.name) }
        return nameArr.joinToString()
    }
    val artistLogo: () -> String? = {
        val logo = CompletableFuture<String?>()
        val headers = mapOf(
                "Authorization" to "Bearer ${BotMusicManager.spotifyToken}"
            )
        BotMusicManager.fetchData("https://api.spotify.com/v1/artists/${artists?.get(0)?.id}", "GET", headers) {
            val images = it.body?.get("images")?.jsonArray
            if (images == null) logo.complete(null)
            if (images?.size!! > 0) {
                val response = images[0].jsonObject["url"]?.jsonPrimitive?.content
                logo.complete(response)
            } else logo.complete(null)
        }
        logo.get()
    }
    val artistURL: String? get() = artists?.get(0)?.external_urls?.spotify
    val cover: String get() = album.images[0].url
    val releaseDate: String get() = album.release_date
    val isrc: String get() = external_ids.isrc

    companion object {
        @Serializable
        data class Artists(@SerialName("name") val name: String,
                           @SerialName("id") val id: String,
                           @SerialName("external_urls") val external_urls: ExternalURLs)

        @Serializable
        data class Album(@SerialName("images") val images: List<Images>, @SerialName("release_date") val release_date: String)

        @Serializable
        data class ExternalURLs(@SerialName("spotify") val spotify: String)

        @Serializable
        data class ExternalIDs(@SerialName("isrc") val isrc: String)

        @Serializable
        data class Images(@SerialName("url") val url: String)
    }
}
