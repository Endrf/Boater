package com.endrf.boater.commands
import com.endrf.boater.BoaterCommand
import com.endrf.boater.BotMusicManager
import com.endrf.boater.embeds.ErrorEmbed
import com.endrf.boater.embeds.QueueListEmbed
import net.dv8tion.jda.api.EmbedBuilder
import net.dv8tion.jda.api.events.interaction.command.CommandAutoCompleteInteractionEvent
import net.dv8tion.jda.api.events.interaction.command.SlashCommandInteractionEvent
import net.dv8tion.jda.api.interactions.commands.Command
import net.dv8tion.jda.api.interactions.commands.OptionType
import net.dv8tion.jda.api.interactions.commands.build.SubcommandData

@Suppress("UNUSED")
class QueueCommand : BoaterCommand(
    "queue",
    "Play, remove, or list items in queue"
) {
    init {
        addSubcommands(
            SubcommandData("list", "List all songs in the server queue ordered by position"),
            SubcommandData("play", "Play a song from the server queue")
                .addOption(OptionType.INTEGER, "queue-number", "Position of the song in the server queue", true, true),
            SubcommandData("remove", "Remove a song from the server queue")
                .addOption(OptionType.INTEGER, "queue-number", "Position of the song in the server queue", true, true)
        )
    }

    override fun execute(interaction: SlashCommandInteractionEvent) {
        val manager = BotMusicManager.getManager(interaction.jda)
        val queue = manager?.getQueue(interaction.guild!!)

        when (interaction.subcommandName) {
            "list" -> interaction.replyEmbeds(QueueListEmbed(queue).build()).queue()
            "play" -> {
                val index = interaction.getOption("queue-number")?.asInt!! - 1
                ErrorEmbed.check(interaction.member?.voiceState?.channel, queue, index)?.let { interaction.replyEmbeds(it).queue(); return; }

                val song = queue?.songs?.get(index)
                interaction.replyEmbeds(
                    EmbedBuilder()
                        .setDescription("### Set song at queue position #${index + 1} (${song?.songs?.get(song.position)?.artists} - ${song?.songs?.get(song.position)?.title}) as Currently Playing")
                        .setColor(0xF4E04C)
                        .build()
                ).queue()
                queue?.forcePlayFromQueue(index)
            }
            "remove" -> {
                val index = interaction.getOption("queue-number")?.asInt!! - 1
                ErrorEmbed.check(interaction.member?.voiceState?.channel, queue, index)?.let { interaction.replyEmbeds(it).queue(); return; }

                val song = queue?.removeSong(index)
                interaction.replyEmbeds(
                    EmbedBuilder()
                        .setDescription("### Removed song at queue position #${index + 1} (${song?.songs?.get(song.position)?.artists} - ${song?.songs?.get(song.position)?.title})")
                        .setColor(0xF4E04C)
                        .build()
                ).queue()
            }
        }
    }

    override fun autoComplete(interaction: CommandAutoCompleteInteractionEvent) {
        if (interaction.focusedOption.name == "queue-number") {
            val songs = BotMusicManager.getManager(interaction.jda)?.getQueue(interaction.guild!!)?.songs
            val options = ArrayList<Command.Choice>()
            songs?.forEachIndexed { index, song ->
                options.add(Command.Choice(
                    ("${index + 1}. ${song.songs[song.position].artists} - ${song.songs[song.position].title}").takeIf { it.length <= 100 }
                        ?: (("${index + 1}. ${song.songs[song.position].artists} - ${song.songs[song.position].title}").take(97) + "..."),
                    index.toLong() + 1))
            }
            interaction.replyChoices(options).queue()
        }
    }
}