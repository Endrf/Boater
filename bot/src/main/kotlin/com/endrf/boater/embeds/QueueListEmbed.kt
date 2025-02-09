package com.endrf.boater.embeds

import com.endrf.boater.MusicQueue
import net.dv8tion.jda.api.EmbedBuilder

class QueueListEmbed(val queue: MusicQueue?) : EmbedBuilder() {
    init {
        when(queue) {
            null -> noQueue()
            else -> queue()
        }
    }

    private fun noQueue() {
        setDescription("## No existing queue :(\nUse `/play` to start a queue")
        setColor(0xF4E04C)
    }

    private fun queue() {
        val queueSize = queue?.songs?.size as Int
        setDescription("## In-Queue: $queueSize ${if (queueSize == 1) "Song" else "Songs"}")
        setFooter("► Currently Playing: ${queue.song.songs[queue.song.position].title}")
        setColor(0xF4E04C)
        queue.songs.forEachIndexed { index, song ->
            addField("", "${index + 1}.\n**${song.playlistData?.title ?: song.songs[song.position].title}**\nAdded by `${song.interaction.userName}`", true)
        }
    }
}