package com.endrf.boater.embeds

import com.endrf.boater.PlaylistContainer
import kotlinx.coroutines.*
import net.dv8tion.jda.api.EmbedBuilder
import net.dv8tion.jda.api.entities.Message
import net.dv8tion.jda.api.entities.MessageEmbed
import net.dv8tion.jda.api.interactions.components.buttons.Button

class LoadingEmbed(private val song: PlaylistContainer, callback: ((Message?) -> Unit)? = null) : EmbedBuilder() {
    private var loaded = false
    init {
        val userInteraction = song.interaction
        var message: Message? = null
        var barPosition = 0
        CoroutineScope(Dispatchers.IO).launch {
            if (userInteraction.interaction != null) {
                with(userInteraction.interaction!!.replyEmbeds(embed(0))) {
                    setActionRow(Button.danger("cancel", "Cancel"))
                    queue { hook -> hook.retrieveOriginal().queue { message = it } }
                }
            } else if (userInteraction.message != null) {
                with(userInteraction.message!!.replyEmbeds(embed(0))) {
                    setActionRow(Button.danger("cancel", "Cancel"))
                    queue { message = it }
                }
            }

            while (!loaded) { // Update animation
                if (barPosition == 11) barPosition = 0
                message?.editMessageEmbeds(embed(barPosition))?.queue()
                barPosition++
                delay(500)
            }

            message?.editMessageComponents()?.queue()
            if (callback != null && message != null) {
                callback(message)
            } else if (callback != null) {
                callback(null)
            }
        }
    }

    private fun embed(barPosition: Int): MessageEmbed {
        setDescription("## **Loading:** [${song.songs[song.position].title}](${song.songs[song.position].url})" +
                "\n### 【${
                    when (barPosition) {
                        0 -> "▰▱▱▱▱▱▱"
                        1 -> "▰▰▰▱▱▱▱"
                        2 -> "▰▰▰▰▰▱▱"
                        3 -> "▰▰▰▰▰▰▰"
                        4 -> "▰▰▰▰▰▰▰"
                        5 -> "▱▱▰▰▰▰▰"
                        6 -> "▱▱▱▱▰▰▰"
                        7 -> "▱▱▱▱▱▱▰"
                        else -> "▱▱▱▱▱▱▱"
                    }
                }】")
        setThumbnail(song.songs[song.position].cover) // Why the hell am I sending a thumbnail multiple times a second???
        setColor(0xF4E04C)
        return build()
    }

    fun finish() {
        loaded = true
    }
}