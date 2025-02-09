package com.endrf.boater.config

import kotlinx.serialization.Serializable
import net.peanuuutz.tomlkt.Toml
import java.io.File
import kotlin.system.exitProcess

object Config {
    private val config = if (File("config.toml").exists()) {
        File("config.toml")
    } else if (File("../config.toml").exists()) {
        File("../config.toml")
    } else {
        println("\u001B[31mError: No config.toml file found")
        exitProcess(1)
    }
    val data: ConfigData = try {
        Toml{ignoreUnknownKeys = true}.decodeFromString(ConfigData.serializer(), config.readText())
    } catch (error: Exception) {
        println("\u001B[31mError: config.toml includes invalid token. Ensure keys are set to correct values.")
        exitProcess(1)
    }
    val botConfig: Bot = data.bot
}

@Serializable
data class ConfigData (
    val bot: Bot
)

@Serializable
data class Bot(
    val enableWebsocket: Boolean,
    val websocketPort: Int,
    val cacheSongs: Boolean,
    val useYtdlp: Boolean,
    val audioFileFormat: String,
    val audioFileQuality: Int
)