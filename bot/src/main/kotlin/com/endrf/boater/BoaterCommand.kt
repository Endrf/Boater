package com.endrf.boater
import net.dv8tion.jda.internal.interactions.CommandDataImpl
import net.dv8tion.jda.api.events.interaction.command.SlashCommandInteractionEvent
import net.dv8tion.jda.api.events.interaction.command.CommandAutoCompleteInteractionEvent

abstract class BoaterCommand(name: String, description: String) : CommandDataImpl(name, description) {
    abstract fun execute(interaction: SlashCommandInteractionEvent)
    open fun autoComplete(interaction: CommandAutoCompleteInteractionEvent) {}
}