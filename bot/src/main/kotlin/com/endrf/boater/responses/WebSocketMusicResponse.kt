package com.endrf.boater.responses

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class WebSocketMusicResponse(
    @SerialName("id") val id: String,
    @SerialName("url") val url: String,
    @SerialName("title") val title: String,
    @SerialName("provider") val provider: String,
    @SerialName("artists") val artists: String,
    @SerialName("cover") val cover: String,
    @SerialName("duration_ms") val durationMs: Int,
    @SerialName("release_date") val releaseDate: String
)
