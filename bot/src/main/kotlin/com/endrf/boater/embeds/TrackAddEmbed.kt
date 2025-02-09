package com.endrf.boater.embeds

import com.endrf.boater.MusicQueue
import com.endrf.boater.PlaylistContainer
import net.dv8tion.jda.api.EmbedBuilder

class TrackAddEmbed(track: PlaylistContainer, queue: MusicQueue) : EmbedBuilder() {
    init {
        val interaction = track.interaction.interaction!!
        setAuthor("Queue Position: ${queue.songs.size}")
        setDescription("### Added [${track.playlistData?.title ?: track.songs[track.position].title}](${track.playlistData ?: track.songs[track.position].url}) to the queue")
        setFooter("Requested by ${interaction.user.globalName}", interaction.user.avatarUrl)
        setColor(0xF4E04C)
    }
}