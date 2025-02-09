package com.endrf.boater

data class PlaylistContainer(
    val songs: ArrayList<MusicApiResponse>,
    val interaction: UserInteraction,
    val playlistData: ServicePlaylist? = null,
    var position: Int = 0,
    var fileName: String? = null
)