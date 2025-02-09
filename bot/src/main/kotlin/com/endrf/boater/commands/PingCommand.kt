package com.endrf.boater.commands
import com.endrf.boater.BoaterCommand
import net.dv8tion.jda.api.events.interaction.command.SlashCommandInteractionEvent

@Suppress("UNUSED")
class PingCommand : BoaterCommand(
    "ping",
    "Ping the bot"
) {
    override fun execute(interaction: SlashCommandInteractionEvent) {
        interaction.reply("I think it works").queue()
    }
}