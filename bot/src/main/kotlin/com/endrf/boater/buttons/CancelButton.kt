package com.endrf.boater.buttons

import com.endrf.boater.BotMusicManager
import net.dv8tion.jda.api.interactions.components.buttons.ButtonInteraction

class CancelButton {
    companion object {
        fun execute(interaction: ButtonInteraction) {
            val queue = BotMusicManager.getManager(interaction.jda)?.getQueue(interaction.guild!!)
            queue?.cancelCurrent = true
        }
    }
}