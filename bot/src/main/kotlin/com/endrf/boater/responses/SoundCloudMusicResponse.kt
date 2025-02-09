package com.endrf.boater.responses

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class SoundCloudMusicResponse(
    @SerialName("id") private val idLong: Long,
    @SerialName("title") val title: String,
    @SerialName("permalink_url") val url: String,
    @SerialName("artwork_url") val artworkURL: String?,
    @SerialName("duration") val durationMs: Int,
    @SerialName("created_at") val createdAt: String,
    @SerialName("user") private val user: User
) {
    val id: String get() = idLong.toString()
    val artist: String get() = user.username
    val artistLogo: String get() = user.avatar_url
    val artistURL: String get() = user.permalink_url
    val cover: String get() = artworkURL ?: ""
    val releaseDate: String get() = createdAt.split('T').first()

    companion object {
        @Serializable
        data class User(
            @SerialName("username") val username: String,
            @SerialName("avatar_url") val avatar_url: String,
            @SerialName("permalink_url") val permalink_url: String
        )
    }
}