package com.endrf.boater.commands

import com.endrf.boater.BoaterCommand
import com.endrf.boater.BotMusicManager
import com.endrf.boater.embeds.ErrorEmbed
import net.dv8tion.jda.api.EmbedBuilder
import net.dv8tion.jda.api.events.interaction.command.SlashCommandInteractionEvent
import net.dv8tion.jda.api.interactions.commands.OptionType

@Suppress("UNUSED")
class VolumeCommand : BoaterCommand(
    "volume",
    "Set the volume of the current server queue"
) {
    init {
        addOption(OptionType.INTEGER, "percent", "A number expressed as a percent excluding the percentage symbol. ex: (98)", true)
    }

    override fun execute(interaction: SlashCommandInteractionEvent) {
        val queue = BotMusicManager.getManager(interaction.jda)?.getQueue(interaction.guild!!)
        ErrorEmbed.check(interaction.member?.voiceState?.channel, queue)?.let { interaction.replyEmbeds(it).queue(); return; }

        val percent = interaction.getOption("percent")?.asInt!!
        interaction.replyEmbeds(
            EmbedBuilder()
                .setDescription("### Set queue volume to $percent%")
                .setColor(0xF4E04C)
                .build()
        ).queue()
        queue?.setVolume(percent)
    }
}