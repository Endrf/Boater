package com.endrf.boater

import kotlinx.serialization.Contextual
import kotlinx.serialization.Serializable
import net.dv8tion.jda.api.entities.Message
import net.dv8tion.jda.api.events.interaction.command.SlashCommandInteractionEvent

@Suppress("PLUGIN_IS_NOT_ENABLED")
@Serializable
data class UserInteraction(
    val id: String,
    val userName: String,
    val avatarUrl: String,
    val guildId: String,
    @Contextual var message: Message? = null,
    @Contextual var interaction: SlashCommandInteractionEvent? = null
) {
    companion object {
        fun create(interaction: SlashCommandInteractionEvent): UserInteraction {
            return UserInteraction(id = interaction.user.id, userName = interaction.user.name, avatarUrl = interaction.user.avatarUrl ?: "", guildId = interaction.guild!!.id, interaction = interaction)
        }
    }
}
