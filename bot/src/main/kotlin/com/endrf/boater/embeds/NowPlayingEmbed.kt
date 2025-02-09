package com.endrf.boater.embeds

import com.endrf.boater.MusicQueue
import com.endrf.boater.PlaylistContainer
import net.dv8tion.jda.api.EmbedBuilder
import java.time.LocalDate
import java.time.format.DateTimeFormatter

class NowPlayingEmbed(track: PlaylistContainer, queue: MusicQueue) : EmbedBuilder() {
    init {
        val song = track.songs[track.position]
        val interaction = track.interaction
        setAuthor("» ${song.artists}", song.artistURL, song.artistLogo)
        setDescription("## **Now Playing: [${song.title}](${song.url})**")
        setThumbnail(song.cover)
        setColor(0xF4E04C)
        setFooter("Requested by ${interaction.userName}", interaction.avatarUrl)
        addField("", "Duration: \n**${calculateTimeToString(song.durationMs)}**", true)
        addField("", "Released: \n**${song.releaseDate?.let { formatUploadDate(it) }}**", true)
        addField("", "Provider: \n**${song.provider}**${if (song.provider == "apple") " *(streaming via spotify)*" else ""}", false)
        if (queue.songs.isNotEmpty()) addField("", "► Up Next: **${queue.songs[0].songs[0].title}**", false)
    }

    private fun calculateTimeToString(durationMs: Int): String {
        val durationSec = durationMs / 1000
        var minutes = 0
        val seconds: Int
        if (durationSec >= 60) {
            minutes = durationSec / 60
            seconds = durationSec - minutes * 60
        } else if (durationSec == 0) {
            minutes = 0
            seconds = 0
        } else {
            seconds = durationSec
        }
        return "${if (minutes > 0) "$minutes minutes, " else ""}${seconds} seconds"
    }

    private fun formatUploadDate(releaseDateString: String): String {
        val currentDate = LocalDate.now()
        val releaseDate = LocalDate.parse(releaseDateString, DateTimeFormatter.ofPattern("yyyy-MM-dd")) // Error with 2013 (yyyy) because not in correct format
        val years = releaseDate.until(currentDate).years
        val months = releaseDate.until(currentDate).months
        val days = releaseDate.until(currentDate).days
        return when {
            years == 1 -> "1 year ago"
            years > 1 -> "$years years ago"
            months == 1 -> "1 month ago"
            months > 1 -> "$months months ago"
            days == 1 -> "1 day ago"
            days > 1 -> "$days days ago"
            else -> "$days days ago"
        }
    }
}