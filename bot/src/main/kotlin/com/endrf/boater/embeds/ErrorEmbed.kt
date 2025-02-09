package com.endrf.boater.embeds

import com.endrf.boater.MusicApiResponse
import com.endrf.boater.MusicQueue
import net.dv8tion.jda.api.EmbedBuilder
import net.dv8tion.jda.api.entities.MessageEmbed
import net.dv8tion.jda.api.entities.channel.unions.AudioChannelUnion

class ErrorEmbed(message: String) : EmbedBuilder() {
    init {
        setTitle("Error: $message")
        setColor(0xF44C4C)
    }

    companion object : EmbedBuilder() {

        fun check(voice: AudioChannelUnion?, musicResponse: MusicApiResponse?): MessageEmbed? { // Used for queue creation
            when {
                voice == null -> return embed("Must be in voice channel")
                musicResponse == null -> return embed("Could not find requested song")
            }
            return null
        }

        fun check(voice: AudioChannelUnion?, queue: MusicQueue?): MessageEmbed? { // Used for skip command
            when {
                voice == null -> return embed("Must be in voice channel")
                queue == null -> return embed("Could not find an active queue for this server")
            }
            return null
        }

        fun check(voice: AudioChannelUnion?, queue: MusicQueue?, queuePosition: Int): MessageEmbed? { // Used for queue play, remove
            when {
                voice == null -> return embed("Must be in voice channel")
                queue == null -> return embed("Could not find an active queue for this server")
                queuePosition + 1 > queue.songs.size || queuePosition < 0 -> return embed("That is not a valid queue position")
            }
            return null
        }

        private fun embed(message: String): MessageEmbed {
            setTitle("Error: $message")
            setColor(0xF44C4C)
            return build()
        }
    }
}