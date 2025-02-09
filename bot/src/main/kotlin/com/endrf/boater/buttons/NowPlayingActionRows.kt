package com.endrf.boater.buttons

import com.endrf.boater.BoaterActionRow
import com.endrf.boater.BotMusicManager
import com.endrf.boater.MusicQueue
import net.dv8tion.jda.api.entities.emoji.Emoji
import net.dv8tion.jda.api.events.interaction.component.ButtonInteractionEvent
import net.dv8tion.jda.api.interactions.components.ActionRow
import net.dv8tion.jda.api.interactions.components.buttons.Button

class NowPlayingActionRows {
    companion object : BoaterActionRow {
        override fun execute(interaction: ButtonInteractionEvent) {
            val queue = BotMusicManager.getManager(interaction.jda)?.getQueue(interaction.guild!!) ?: return
            when(interaction.button.id) {
                "pausePlay" -> {
                    queue.togglePausePlay()
                    interaction.editComponents(ActionRow.of(controllerMain(queue))).queue()
                }
                "skip" -> {
                    queue.forcePlayFromQueue()
                }
            }
        }

        fun controllerMain(queue: MusicQueue): List<Button> {
            val pausePlay =
                if (!queue.isPlayerInit() || !queue.player.isPaused)
                    Button.primary("pausePlay", "Pause").withEmoji(Emoji.fromUnicode("\u23F8"))
                else
                    Button.primary("pausePlay", "Play").withEmoji(Emoji.fromUnicode("\u25B6"))
            val loop =
                if (queue.loop)
                    Button.success("loop", "Loop").withEmoji(Emoji.fromUnicode("U+1F502"))
                else
                    Button.secondary("loop", "Loop").withEmoji(Emoji.fromUnicode("U+1F502"))
            return listOf(
                pausePlay,
                Button.danger("skip", "Skip").withEmoji(Emoji.fromUnicode("\u23E9")),
                loop
            )
        }

        fun controllerExtra(): List<Button> {
            return listOf(
            )
        }
    }
}