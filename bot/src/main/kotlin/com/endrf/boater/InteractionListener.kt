package com.endrf.boater
import com.endrf.boater.buttons.CancelButton
import net.dv8tion.jda.api.events.interaction.command.SlashCommandInteractionEvent
import net.dv8tion.jda.api.events.interaction.command.CommandAutoCompleteInteractionEvent
import net.dv8tion.jda.api.events.interaction.component.ButtonInteractionEvent
import net.dv8tion.jda.api.hooks.ListenerAdapter

class InteractionListener(private val commands: ArrayList<BoaterCommand>) : ListenerAdapter() {

    override fun onSlashCommandInteraction(interaction: SlashCommandInteractionEvent) {
        val index = commands.indexOfFirst{it.name == interaction.name}
        if (index != -1) commands[index].execute(interaction)
    }

    override fun onCommandAutoCompleteInteraction(interaction: CommandAutoCompleteInteractionEvent) {
        val index = commands.indexOfFirst { it.name == interaction.name }
        if (index != -1) commands[index].autoComplete(interaction)
    }

    override fun onButtonInteraction(interaction: ButtonInteractionEvent) {
        when(interaction.button.id) {
            "cancel" -> CancelButton.execute(interaction)
            else -> BotMusicManager.messageButtonIdMap[interaction.messageId]?.execute(interaction)
        }
    }
}