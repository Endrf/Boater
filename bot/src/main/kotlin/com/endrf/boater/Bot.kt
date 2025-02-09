package com.endrf.boater
import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver
import com.endrf.BoaterData
import com.endrf.boater.config.Config
import com.endrf.boater.responses.WebSocketMusicResponse
import com.squareup.moshi.Moshi
import com.squareup.moshi.adapter
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import io.github.cdimascio.dotenv.dotenv
import io.ktor.server.application.*
import io.ktor.server.engine.*
import io.ktor.server.netty.*
import io.ktor.server.routing.*
import io.ktor.server.websocket.*
import io.ktor.websocket.*
import io.ktor.serialization.kotlinx.*
import net.dv8tion.jda.api.JDA
import net.dv8tion.jda.api.JDABuilder
import net.dv8tion.jda.api.requests.GatewayIntent
import net.dv8tion.jda.api.entities.Activity
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.*
import org.reflections.Reflections
import org.reflections.scanners.Scanners
import java.time.Duration
import java.util.*
import java.util.stream.Collectors
import kotlin.collections.ArrayList
import kotlin.collections.LinkedHashMap

val dotenv = dotenv { directory = "../" }

fun main(args: Array<String>) {
    val dbDriver: SqlDriver = JdbcSqliteDriver("jdbc:sqlite:boater_data.db", Properties().apply { put("foreign_keys", "true") })
    val commands = getCommands()
    val jda: JDA = runBlocking {
        JDABuilder.createDefault(dotenv["DISCORD_DEVELOPER_KEY"])
            .enableIntents(GatewayIntent.GUILD_MEMBERS, GatewayIntent.GUILD_VOICE_STATES, GatewayIntent.MESSAGE_CONTENT)
            .setActivity(Activity.playing("Absolute Bangers"))
            .addEventListeners(InteractionListener(commands) /* Only need one listener */)
            .build().awaitReady()
    }
    BoaterData.Schema.create(dbDriver)
    val database = BoaterData(dbDriver)
    BotMusicManager(jda)
    println("\u001b[32mLoaded! Logged in as ${jda.selfUser.name}#${jda.selfUser.discriminator}\u001B[0m")

    if (args.contains("--update-guild")) {
        val index = args.indexOfFirst {it == "--update-guild"} + 1
        if (index < args.size) {
            try {
                val guildId: Long = args[index].toLong()
                val guild = jda.getGuildById(guildId)
                if (guild != null) {
                    println("\u001B[33m[Boater Command Handler] \u001B[0mStarted updating ${commands.size} application (/) commands for ${guild.name}")
                } else {
                    println("\u001B[33m[Boater] \u001b[31mError: Could not find guild\u001B[0m")
                }
                guild?.updateCommands()?.addCommands(commands)?.queue {
                    println("\u001B[33m[Boater Command Handler] \u001B[0mSuccessfully updated ${commands.size} application (/) commands for ${guild.name}")
                }
            }
            catch (err: NumberFormatException) {
                println("\u001B[33m[Boater] \u001b[31mError: Guild ID must be a number\u001B[0m")
            }
        }
    }

    if (Config.botConfig.enableWebsocket) {
        startWebsocket(jda, database)
    }
}

fun getCommands(): ArrayList<BoaterCommand> {
    val commands = ArrayList<BoaterCommand>() // List of all Command Class Instances
    Reflections("com.endrf.boater.commands", Scanners.SubTypes) // Get all Class objects from commands package and push instance to array
        .getSubTypesOf(BoaterCommand::class.java)
        .stream().collect(Collectors.toList())
        .forEach {
            val inst = it.getDeclaredConstructor().newInstance()
            commands.add(inst)
            println("\u001B[33m[Boater Command Handler] \u001B[0m${inst.name} Command has been loaded")
        }
    return commands
}

@OptIn(ExperimentalStdlibApi::class)
fun startWebsocket(jda: JDA, database: BoaterData) {
    embeddedServer(Netty, configure = {
        requestQueueLimit = 16
        connectionGroupSize = 2
        workerGroupSize = 2
        callGroupSize = 2
        shareWorkGroup = true
        responseWriteTimeoutSeconds = 10
    }, port = Config.botConfig.websocketPort) {
        install(WebSockets) {
            contentConverter = KotlinxWebsocketSerializationConverter(Json { explicitNulls = false })
            pingPeriod = Duration.ofSeconds(15)
            timeout = Duration.ofSeconds(30)
            maxFrameSize = Long.MAX_VALUE
            masking = false
        }
        println("\u001B[33m[Boater] \u001B[0mWebsocket listening on port ${Config.botConfig.websocketPort}")
        routing {
            val clients = LinkedHashMap<DefaultWebSocketServerSession, String>()
            val moshi = Moshi.Builder().addLast(KotlinJsonAdapterFactory()).build()
            webSocket("/guild") {
                try {
                    for (frame in incoming) {
                        val request = Json.decodeFromString<WebSocketPayload>((frame as Frame.Text).readText())
                        when (request.action) {
                            "setGuild" -> {
                                if (request.data == null) return@webSocket
                                try {
                                    sendSerialized("[connected]")
                                    sendSerialized(BotMusicManager.getManager(jda)?.getQueue(request.data)?.initialData())
                                } catch (e: Exception) {
                                    println(e.message)
                                    e.printStackTrace()
                                }
                                println(request.data)
                                clients[this] = request.data
                                println(clients.size)
                            }
                            "ping" -> {
                                sendSerialized(mapOf("db_response" to false))
                            }
                            "play" -> {
                                if (request.dataObject == null) return@webSocket
                                val guildId = clients[this]
                                val url = request.dataObject["url"]!!.jsonPrimitive.content
                                val provider = request.dataObject["provider"]!!.jsonPrimitive.content
                                val user = Json.decodeFromJsonElement<UserInteraction>(request.dataObject["user"]!!)
                                CoroutineScope(Dispatchers.IO).launch {
                                    guildId?.let { BotMusicManager.getManager(jda)?.playSong(url, provider, user) }
                                }
                            }
                            "playPlaylist" -> {
                                if (request.dataObject == null) return@webSocket
                                val data = request.dataObject
                                val guildId = clients[this]
                                val playlist = moshi.adapter<ServicePlaylist>().fromJson(data["playlist"]!!.jsonObject.toString())!!
                                val user = Json.decodeFromJsonElement<UserInteraction>(data["user"]!!)
                                val songs = data["songs"]!!.jsonArray.map { element ->
                                    MusicApiResponse.convertToMusicResponse(Json.decodeFromJsonElement<WebSocketMusicResponse>(element))
                                } as ArrayList
                                val playlistContainer = PlaylistContainer(songs, user, playlist)
                                CoroutineScope(Dispatchers.IO).launch {
                                    guildId?.let { BotMusicManager.getManager(jda)?.playSong(provider = playlist.provider, userInteraction = user, playlistData = playlistContainer) }
                                }
                            }
                            "togglePlay" -> {
                                val guildId = clients[this]
                                guildId?.let { BotMusicManager.getManager(jda)?.getQueue(it)?.togglePausePlay() }
                            }
                            "skip" -> {
                                val guildId = clients[this]
                                guildId?.let { BotMusicManager.getManager(jda)?.getQueue(it)?.forcePlayFromQueue() }
                            }
                            "skipInPlaylist" -> {
                                val guildId = clients[this]
                                guildId?.let { BotMusicManager.getManager(jda)?.getQueue(it)?.forcePlayFromQueue(inPlaylist = true) }
                            }
                            "forcePlay" -> {
                                val guildId = clients[this]
                                val index = request.data?.toIntOrNull() ?: return@webSocket
                                guildId?.let { BotMusicManager.getManager(jda)?.getQueue(it)?.forcePlayFromQueue(index) }
                            }
                            "forcePlayInPlaylist" -> {
                                val guildId = clients[this]
                                val index = request.data?.toIntOrNull() ?: return@webSocket
                                guildId?.let { BotMusicManager.getManager(jda)?.getQueue(it)?.forcePlayFromQueue(index, inPlaylist = true) }
                            }
                            "remove" -> {
                                val guildId = clients[this]
                                val index = request.data?.toIntOrNull() ?: return@webSocket
                                guildId?.let { BotMusicManager.getManager(jda)?.getQueue(it)?.removeSong(index) }
                            }
                            "getUsers" -> {
                                sendSerialized(Json.parseToJsonElement(moshi.adapter<Map<String,Any>>().toJson(mapOf(
                                    "db_data" to mapOf("users" to database.usersQueries.getUsers().executeAsList()),
                                    "db_response" to false)
                                )).jsonObject)
                            }
                            "getPlaylistExists" -> {
                                if (request.dataObject == null) return@webSocket
                                val id = request.dataObject["id"]?.jsonPrimitive?.content ?: return@webSocket
                                val owner = request.dataObject["owner"]?.jsonPrimitive?.content ?: return@webSocket
                                sendSerialized(mapOf("db_response" to Json.parseToJsonElement(moshi.adapter<Playlists>().toJson(database.playlistsQueries.getPlaylist(id, owner).executeAsOneOrNull()))))
                            }
                            "addUserPlaylist" -> {
                                if (request.dataObject == null) return@webSocket
                                val data = request.dataObject
                                val user = moshi.adapter<Users>().fromJson(data["user"]!!.jsonObject.toString())!!
                                val playlist = moshi.adapter<ServicePlaylist>().lenient().fromJson(data["playlist"]!!.jsonObject.toString())!!
                                val songs = moshi.adapter<List<Songs>>().fromJson(data["songs"]!!.jsonArray.toString())!!
                                database.transaction {
                                    if (!database.usersQueries.getUserExists(user.id, user.display_name, user.avatar).executeAsOne()) { // Insert user if it doesn't exist, replace if a column changes
                                        database.usersQueries.addUser(user)
                                    }
                                    database.playlistsQueries.addPlaylist(playlist.id, playlist.provider, playlist.title, playlist.artist, playlist.cover, playlist.song_count, user.id) // Insert or replace playlist
                                    if (database.playlistsQueries.getPlaylistExists(playlist.id, user.id).executeAsOne()) {
                                        database.playlistsQueries.deleteSongRelations(playlist.id, user.id) // Delete old song relations
                                    }

                                    for (item in songs) { // Insert songs and song relations
                                        database.songsQueries.addSong(item)
                                        database.playlistsQueries.addSongRelation(playlist.id, user.id, item.url)
                                    }
                                }
                            }
                            "removeUserPlaylist" -> {
                                if (request.dataObject == null) return@webSocket
                                val id = request.dataObject["id"]?.jsonPrimitive?.content ?: return@webSocket
                                val owner = request.dataObject["owner"]?.jsonPrimitive?.content ?: return@webSocket
                                database.transaction {
                                    database.playlistsQueries.deletePlaylist(id, owner)
                                    database.usersQueries.deleteUser(owner, owner)
                                }
                                sendSerialized(mapOf("db_response" to false))
                            }
                            "addPlaylistSongs" -> {
                                if (request.dataObject == null) return@webSocket
                                val user = moshi.adapter<Users>().fromJson(request.dataObject["user"]!!.jsonObject.toString())!!
                                val songs = moshi.adapter<List<Songs>>().fromJson(request.dataObject["songs"]!!.jsonArray.toString())!!
                                val playlistId = request.dataObject["id"]!!.jsonPrimitive.content
                                var newId = playlistId
                                database.transaction {
                                    // !!!May have issue if there are multiple playlists with same id but different owners
                                    // might be solved using id in user dataObject

                                    val plSongs = ArrayList(database.playlistsQueries.getSongRelations(playlistId, user.id).executeAsList())
                                    database.playlistsQueries.deleteSongRelations(playlistId, user.id) // Delete old song relations
                                    if (request.dataObject["convert"]?.jsonPrimitive?.boolean == true) {
                                        // update playlist and id to boater type
                                        do newId = "b." + UUID.randomUUID().toString().substring(0, 7)
                                        while (database.playlistsQueries.getPlaylistIdExists(newId).executeAsOneOrNull() == true)

                                        database.playlistsQueries.updatePlaylistIdToBoater(newId, user.display_name, playlistId)
                                    }

                                    plSongs.addAll(request.dataObject["position"]!!.jsonPrimitive.int, songs)
                                    // song_count is an unnecessary field. Should count relations in db to determine song count.
                                    database.playlistsQueries.updatePlaylistSongCount(plSongs.size.toLong(), newId)
                                    for (item in plSongs) { // Insert songs and song relations
                                        database.songsQueries.addSong(item)
                                        database.playlistsQueries.addSongRelation(newId, user.id, item.url)
                                    }
                                }
                                if (request.dataObject["convert"]?.jsonPrimitive?.boolean == true) {
                                    sendSerialized(mapOf("db_response" to mapOf("newId" to newId)))
                                } else {
                                    sendSerialized(mapOf("db_response" to false))
                                }
                            }
                            "removePlaylistSong" -> {
                                if (request.dataObject == null) return@webSocket
                                val user = moshi.adapter<Users>().fromJson(request.dataObject["user"]!!.jsonObject.toString())!!
                                val position = request.dataObject["position"]!!.jsonPrimitive.int
                                val playlistId = request.dataObject["id"]!!.jsonPrimitive.content
                                var newId = playlistId
                                database.transaction {
                                    val plSongs = ArrayList(database.playlistsQueries.getSongRelations(playlistId, user.id).executeAsList())
                                    database.playlistsQueries.deleteSongRelations(playlistId, user.id)
                                    if (request.dataObject["convert"]?.jsonPrimitive?.boolean == true) {
                                        // update playlist and id to boater type
                                        //newId = "b." + UUID.randomUUID().toString().substring(0, 7)
                                        do newId = "b." + UUID.randomUUID().toString().substring(0, 7)
                                        while (database.playlistsQueries.getPlaylistIdExists(newId).executeAsOneOrNull() == true)

                                        database.playlistsQueries.updatePlaylistIdToBoater(newId, user.display_name, playlistId)
                                    }

                                    plSongs.removeAt(position)
                                    database.playlistsQueries.updatePlaylistSongCount(plSongs.size.toLong(), newId)
                                    for (item in plSongs) {
                                        database.songsQueries.addSong(item)
                                        database.playlistsQueries.addSongRelation(newId, user.id, item.url)
                                    }
                                }
                                if (request.dataObject["convert"]?.jsonPrimitive?.boolean == true) {
                                    sendSerialized(mapOf("db_response" to mapOf("newId" to newId)))
                                } else {
                                    sendSerialized(mapOf("db_response" to false))
                                }
                            }
                            "getUserPlaylists" -> {
                                sendSerialized(Json.parseToJsonElement(moshi.adapter<Map<String,Any>>().toJson(mapOf(
                                    "db_data" to mapOf("userPlaylists" to database.playlistsQueries.getUserPlaylists(request.data!!).executeAsList()),
                                    "db_response" to true)
                                )).jsonObject)
                            }
                            "getPlaylistSongs" -> {
                                if (request.dataObject == null) return@webSocket
                                val id = request.dataObject["id"]?.jsonPrimitive?.content ?: return@webSocket
                                val owner = request.dataObject["owner"]?.jsonPrimitive?.content ?: return@webSocket
                                sendSerialized(mapOf("db_data" to DatabasePayload(
                                    playlistSongs = Json.parseToJsonElement(moshi.adapter<List<Songs>>().toJson(database.playlistsQueries.getSongRelations(id, owner).executeAsList())).jsonArray
                                )))
                            }
                        }
                    }
                } catch (e: ClosedReceiveChannelException) {
                    println("socket close")
                } catch (e: Throwable) {
                    println("Error: ${e.message}")
                    e.printStackTrace()
                } finally {
                    clients.remove(this)
                    println(clients.size)
                }
            }
            BotMusicManager.getManager(jda)?.queues?.addListener { musicQueue ->
                val queueClients = { clients.filter { it.value == musicQueue.guild.id } }

                musicQueue.addListener { data ->
                    CoroutineScope(Dispatchers.IO).launch {
                        queueClients().forEach {
                            if (Json.encodeToJsonElement(data).jsonObject.isEmpty()) it.key.send("null")
                            else it.key.sendSerialized(data)
                        }
                    }
                }
            }
        }
    }.start(wait = true)
}

@Suppress("PLUGIN_IS_NOT_ENABLED")
@Serializable
data class WebSocketPayload(val action: String, val data: String? = null, val dataObject: JsonObject? = null)

@Serializable
data class DatabasePayload(val users: JsonArray? = null, val userPlaylists: JsonArray? = null, val playlistSongs: JsonArray? = null)

data class ServicePlaylist(val id: String, val title: String, val artist: String, val cover: String? = null, val song_count: Long, val provider: String)