package com.endrf.boater

import com.endrf.boater.responses.*
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Suppress("PLUGIN_IS_NOT_ENABLED")
@Serializable
data class MusicApiResponse(
    val provider: String,
    @SerialName("title") val title: String,
    @SerialName("id") val id: String,
    @SerialName("artist") val artist: String,
    @SerialName("artists") val artists: String,
    @SerialName("artistLogo") val artistLogo: String? = null,
    @SerialName("artistURL") val artistURL: String? = null,
    @SerialName("url") val url: String,
    @SerialName("cover") val cover: String,
    @SerialName("durationMs") val durationMs: Int,
    @SerialName("releaseDate") val releaseDate: String? = null,
    @SerialName("isrc") val isrc: String? = null
) {
    companion object {
        fun convertToMusicResponse(spotifyMusicResponse: SpotifyMusicResponse): MusicApiResponse {
            return MusicApiResponse(
                provider = "spotify",
                title = spotifyMusicResponse.title,
                id = spotifyMusicResponse.id,
                artist = spotifyMusicResponse.artist,
                artists = spotifyMusicResponse.artistList,
                artistLogo = spotifyMusicResponse.artistLogo(),
                artistURL = spotifyMusicResponse.artistURL,
                url = spotifyMusicResponse.url,
                cover = spotifyMusicResponse.cover,
                durationMs = spotifyMusicResponse.durationMs,
                releaseDate = spotifyMusicResponse.releaseDate,
                isrc = spotifyMusicResponse.isrc
            )
        }

        fun convertToMusicResponse(appleMusicResponse: AppleMusicResponse): MusicApiResponse {
            return MusicApiResponse(
                provider = "apple",
                title = appleMusicResponse.title,
                id = appleMusicResponse.id,
                artist = appleMusicResponse.artist,
                artists = appleMusicResponse.artistList,
                artistLogo = appleMusicResponse.artistLogo(),
                artistURL = appleMusicResponse.artistURL,
                url = appleMusicResponse.url,
                cover = appleMusicResponse.cover,
                durationMs = appleMusicResponse.durationMs,
                releaseDate = appleMusicResponse.releaseDate,
                isrc = appleMusicResponse.isrc
            )
        }

        fun convertToMusicResponse(youtubeMusicResponse: YoutubeMusicResponse): MusicApiResponse {
            return MusicApiResponse(
                provider = "youtube",
                title = youtubeMusicResponse.title,
                id = youtubeMusicResponse.id,
                artist = youtubeMusicResponse.artist,
                artists = youtubeMusicResponse.artist,
                artistLogo = youtubeMusicResponse.artistLogo(),
                artistURL = youtubeMusicResponse.artistURL(),
                url = youtubeMusicResponse.url,
                cover = youtubeMusicResponse.cover,
                durationMs = youtubeMusicResponse.durationMs,
                releaseDate = youtubeMusicResponse.releaseDate,
                isrc = null
            )
        }

        fun convertToMusicResponse(soundCloudMusicResponse: SoundCloudMusicResponse): MusicApiResponse {
            return MusicApiResponse(
                provider = "soundcloud",
                title = soundCloudMusicResponse.title,
                id = soundCloudMusicResponse.id,
                artist = soundCloudMusicResponse.artist,
                artists = soundCloudMusicResponse.artist,
                artistLogo = soundCloudMusicResponse.artistLogo,
                artistURL = soundCloudMusicResponse.artistURL,
                url = soundCloudMusicResponse.url,
                cover = soundCloudMusicResponse.cover,
                durationMs = soundCloudMusicResponse.durationMs,
                releaseDate = soundCloudMusicResponse.releaseDate,
                isrc = null
            )
        }

        fun convertToMusicResponse(webSocketMusicResponse: WebSocketMusicResponse): MusicApiResponse {
            return MusicApiResponse(
                provider = webSocketMusicResponse.provider,
                title = webSocketMusicResponse.title,
                id = webSocketMusicResponse.id,
                artist = webSocketMusicResponse.artists,
                artists = webSocketMusicResponse.artists,
                url = webSocketMusicResponse.url,
                cover = webSocketMusicResponse.cover,
                durationMs = webSocketMusicResponse.durationMs,
                releaseDate = webSocketMusicResponse.releaseDate,
            )
        }
    }
}
