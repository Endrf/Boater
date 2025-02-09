package com.endrf.boater.commands

import com.endrf.boater.BoaterCommand
import com.endrf.boater.BotMusicManager
import com.endrf.boater.embeds.ErrorEmbed
import net.dv8tion.jda.api.EmbedBuilder
import net.dv8tion.jda.api.events.interaction.command.SlashCommandInteractionEvent

@Suppress("UNUSED")
class ResumeCommand : BoaterCommand(
    "resume",
    "Resumes playback of the currently playing song"
) {
    override fun execute(interaction: SlashCommandInteractionEvent) {
        val queue = BotMusicManager.getManager(interaction.jda)?.getQueue(interaction.guild!!)
        ErrorEmbed.check(interaction.member?.voiceState?.channel, queue)?.let { interaction.replyEmbeds(it).queue(); return; }

        interaction.replyEmbeds(
            EmbedBuilder()
                .setDescription("### Resumed ${queue?.song?.songs?.get(queue.song.position)?.artists} - ${queue?.song?.songs?.get(queue.song.position)?.title}")
                .setColor(0xF4E04C)
                .build()
        ).queue()
        queue?.togglePausePlay()
    }
}