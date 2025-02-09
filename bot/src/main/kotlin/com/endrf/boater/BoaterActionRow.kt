package com.endrf.boater

import net.dv8tion.jda.api.events.interaction.component.ButtonInteractionEvent

interface BoaterActionRow {
    fun execute(interaction: ButtonInteractionEvent)
}