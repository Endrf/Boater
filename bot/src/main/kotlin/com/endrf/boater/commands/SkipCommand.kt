package com.endrf.boater.commands

import com.endrf.boater.BoaterCommand
import com.endrf.boater.BotMusicManager
import com.endrf.boater.embeds.ErrorEmbed
import net.dv8tion.jda.api.EmbedBuilder
import net.dv8tion.jda.api.events.interaction.command.SlashCommandInteractionEvent

@Suppress("UNUSED")
class SkipCommand : BoaterCommand(
    "skip",
    "Skip the currently playing song to the next song in the server queue"
) {
    override fun execute(interaction: SlashCommandInteractionEvent) {
        val queue = BotMusicManager.getManager(interaction.jda)?.getQueue(interaction.guild!!)
        ErrorEmbed.check(interaction.member?.voiceState?.channel, queue)?.let { interaction.replyEmbeds(it).queue(); return; }

        interaction.replyEmbeds(
            EmbedBuilder()
                .setDescription("### Skipping to the next song in queue")
                .setColor(0xF4E04C)
                .build()
        ).queue()
        queue?.forcePlayFromQueue(inPlaylist = true)
    }
}