package com.endrf.boater.commands
import com.endrf.boater.BoaterCommand
import com.endrf.boater.BotMusicManager
import com.endrf.boater.UserInteraction
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.dv8tion.jda.api.events.interaction.command.SlashCommandInteractionEvent
import net.dv8tion.jda.api.events.interaction.command.CommandAutoCompleteInteractionEvent
import net.dv8tion.jda.api.interactions.commands.build.OptionData
import net.dv8tion.jda.api.interactions.commands.OptionType
import net.dv8tion.jda.api.entities.channel.ChannelType
import net.dv8tion.jda.api.interactions.commands.Command

@Suppress("UNUSED")
class PlayCommand : BoaterCommand(
    "play",
    "Play a song given a search/url and provider (Adds to queue if a song is already playing)"
) {
    init {
        addOptions(
            OptionData(OptionType.STRING, "search-provider", "The platform to play the song from (Apple Music streams from Spotify)", true)
                .addChoice("Spotify", "spotify")
                .addChoice("Apple Music", "apple")
                .addChoice("Youtube", "youtube")
                .addChoice("SoundCloud", "soundcloud")
                .addChoice("Deezer", "deezer"),
            OptionData(OptionType.STRING, "song", "Title or url of the song", true, true),
            OptionData(OptionType.CHANNEL, "voice-channel", "Specify the voice channel to join")
                .setChannelTypes(ChannelType.VOICE, ChannelType.STAGE)
        )
    }

    override fun execute(interaction: SlashCommandInteractionEvent) {
        val manager = BotMusicManager.getManager(interaction.jda)
        CoroutineScope(Dispatchers.IO).launch {
            manager?.playSong(
                interaction.getOption("song")?.asString as String,
                interaction.getOption("search-provider")?.asString as String,
                UserInteraction.create(interaction)
            )
        }
    }

    override fun autoComplete(interaction: CommandAutoCompleteInteractionEvent) {
        val focusedOption = interaction.focusedOption
        val provider: String? = interaction.getOption("search-provider")?.asString

        if (focusedOption.name == "song") {
            if (provider == null) return
            if (focusedOption.value != "") {
                val results = BotMusicManager.querySong(focusedOption.value, provider, 8)
                val options = ArrayList<Command.Choice>()
                results?.forEach { song ->
                    options.add(Command.Choice(
                    ("${song.artists} - ${song.title}").takeIf { it.length <= 100 }
                        ?: (("${song.artists} - ${song.title}").take(97) + "..."),
                    song.url))
                }
                interaction.replyChoices(options).queue()
            }
        }
    }
}