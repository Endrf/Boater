package com.endrf.boater

import com.endrf.boater.config.Config
import com.endrf.boater.embeds.*
import com.endrf.boater.responses.AppleMusicResponse
import com.endrf.boater.responses.SoundCloudMusicResponse
import com.endrf.boater.responses.SpotifyMusicResponse
import com.endrf.boater.responses.YoutubeMusicResponse
import com.sedmelluq.discord.lavaplayer.player.AudioLoadResultHandler
import com.sedmelluq.discord.lavaplayer.player.AudioPlayer
import com.sedmelluq.discord.lavaplayer.player.DefaultAudioPlayerManager
import com.sedmelluq.discord.lavaplayer.source.AudioSourceManagers
import com.sedmelluq.discord.lavaplayer.tools.FriendlyException
import com.sedmelluq.discord.lavaplayer.track.AudioPlaylist
import com.sedmelluq.discord.lavaplayer.track.AudioTrack
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import kotlinx.coroutines.*
import kotlinx.coroutines.future.await
import kotlinx.serialization.encodeToString
import net.dv8tion.jda.api.JDA
import kotlinx.serialization.json.*
import net.dv8tion.jda.api.entities.Guild
import net.dv8tion.jda.api.entities.Message
import net.dv8tion.jda.api.entities.channel.unions.AudioChannelUnion
import net.dv8tion.jda.api.interactions.components.ActionRow
import net.dv8tion.jda.api.interactions.components.buttons.Button
import java.util.concurrent.CompletableFuture
import java.util.Base64
import java.io.File
import java.net.URLEncoder
import okhttp3.*
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Headers.Companion.toHeaders
import okio.IOException
import java.time.Duration
import kotlin.coroutines.resume
import kotlin.coroutines.suspendCoroutine
import kotlin.io.path.*

class BotMusicManager(private val client: JDA) {
    val queues = ObservableArrayList<MusicQueue>()

    init {
        newManager(this)
        println("\u001B[33m[Boater] \u001B[0m${client.selfUser.name} Music Manager Registered")
    }

    suspend fun playSong(song: String? = null, provider: String, userInteraction: UserInteraction, playlistData: PlaylistContainer? = null) { // Creates queue given song
        val guild = client.getGuildById(userInteraction.guildId)
        println(userInteraction.userName)
        val channel = guild?.getMemberById(userInteraction.id)?.voiceState?.channel
        val musicResponse = if (song != null) {
            arrayListOf(querySong(song, provider)?.get(0)!!)
        } else playlistData?.songs
        ErrorEmbed.check(channel, musicResponse?.get(0))?.let { userInteraction.interaction?.replyEmbeds(it)?.queue(); return; }

        getQueue(userInteraction.guildId)?.addSong(PlaylistContainer(musicResponse!!, userInteraction, playlistData?.playlistData))
            ?: run {
                val queue = MusicQueue(PlaylistContainer(musicResponse!!, userInteraction, playlistData?.playlistData), guild!!, channel)
                queues.add(queue)
                queue.initialize() ?: queues.remove(queue)
            }
    }

    fun getQueue(guild: String): MusicQueue? {
        val index = this.queues.indexOfFirst { it.guild.id == guild }
        if (index == -1) return null
        return queues[index]
    }

    fun getQueue(guild: Guild): MusicQueue? {
        val index = this.queues.indexOfFirst { it.guild == guild }
        if (index == -1) return null
        return queues[index]
    }

    fun removeQueue(queue: MusicQueue) {
        queues.remove(queue)
    }

    companion object {
        private val json = Json { ignoreUnknownKeys = true }
        val moshi = Moshi.Builder().addLast(KotlinJsonAdapterFactory()).build()
        private val okHttpClient = OkHttpClient.Builder()
            .connectTimeout(Duration.ofSeconds(60))
            .writeTimeout(Duration.ofSeconds(60))
            .readTimeout(Duration.ofSeconds(60)).build()
        lateinit var spotifyToken: String
        var appleToken: String = dotenv["APPLE_MUSIC_DEVELOPER_KEY"]
        private val managers = ArrayList<BotMusicManager>()
        val playerManager = DefaultAudioPlayerManager()
        val messageButtonIdMap = HashMap<String, BoaterActionRow>()
        private val audioSources = mapOf(
            "yt-dlp" to mapOf(
                "name" to "yt-dlp (YouTube-Internal)",
                "type" to "internal",
                "load" to { song: MusicApiResponse ->
                    val songPath = createAudioFileString(song.artist, song.title)
                    if (Path(songPath).notExists()) {
                        ProcessBuilder(
                            "yt-dlp",
                            "--extract-audio",
                            "--audio-format", Config.botConfig.audioFileFormat,
                            "--audio-quality", Config.botConfig.audioFileQuality.toString(),
                            "--force-overwrites",
                            "--post-overwrites",
                            "--output", songPath,
                            song.url
                        ).redirectOutput(ProcessBuilder.Redirect.INHERIT).start().waitFor()
                    }
                    songPath
                },
                "enabled" to Config.botConfig.useYtdlp
            ),
            "ytiz" to mapOf(
                "name" to "YTiz.xyz (YouTube-External)",
                "type" to "external",
                "url" to { _: MusicApiResponse -> "https://m9.fly.dev/api/download"},
                "method" to "POST",
                "headers" to mapOf(
                    "Origin" to "https://ytiz.xyz",
                    "Referer" to "https://ytiz.xyz/",
                    "Priority" to "u=0",
                    "Content-Type" to "application/json"
                ),
                "body" to { musicResponse: MusicApiResponse ->
                    Json.encodeToString(JsonObject(mapOf(
                        "endTime" to JsonPrimitive(0),
                        "startTime" to JsonPrimitive(0),
                        "format" to JsonPrimitive("mp3"),
                        "url" to JsonPrimitive(musicResponse.url),
                        "quality" to JsonPrimitive("128"),
                        "randID" to JsonPrimitive(89),
                        "filename" to JsonPrimitive("temporary_89/${musicResponse.title.replace("/", "⧸")}.mp3"),
                        "metadata" to JsonPrimitive(false),
                        "trim" to JsonPrimitive(false)
                    )))
                },
                "processResponse" to { body: Map<String, JsonElement> ->
                    val audioData = CompletableFuture<String?>()
                    val headers = mapOf(
                        "Origin" to "https://ytiz.xyz",
                        "Referer" to "https://ytiz.xyz/",
                        "Priority" to "u=0",
                        "Content-Type" to "application/json"
                    )
                    fetchData("https://m9.fly.dev/api/file_send", "POST", headers, Json.encodeToString(body), bytes = true) {
                        audioData.complete(it.body?.get("bytes")?.jsonPrimitive?.content)
                    }
                    val file = File("stream.mp3")
                    if (audioData.get() == null) audioData.get()
                    else {
                        file.writeBytes(Base64.getDecoder().decode(audioData.get()))
                        "stream.mp3"
                    }
                },
                "plainResponse" to false,
                "enabled" to true
            )
        )
        private val providerSourceSelection = mapOf(
            "spotify" to arrayOf(
                mapOf(
                    "default" to false,
                    "source" to audioSources["yt-dlp"],
                    "convert" to { musicResponse: MusicApiResponse ->
                        if (musicResponse.isrc == null) {
                            querySong("${musicResponse.artist} ${musicResponse.title}", "youtube")?.get(0)
                        } else {
                            querySong(musicResponse.isrc, "youtube")?.get(0) ?:
                            querySong("${musicResponse.artist} ${musicResponse.title}", "youtube")?.get(0)
                        }
                    },
                    "attempts" to 2
                ),
                mapOf(
                    "default" to true,
                    "convert" to { musicResponse: MusicApiResponse ->
                        querySong("${musicResponse.artists} ${musicResponse.title}", "soundcloud")?.get(0)
                    }
                ),
                mapOf(
                    "default" to false,
                    "source" to audioSources["ytiz"],
                    "convert" to { musicResponse: MusicApiResponse ->
                        if (musicResponse.isrc == null) {
                            querySong("${musicResponse.artist} ${musicResponse.title}", "youtube")?.get(0)
                        } else {
                            querySong(musicResponse.isrc, "youtube")?.get(0) ?:
                            querySong("${musicResponse.artist} ${musicResponse.title}", "youtube")?.get(0)
                        }
                    },
                    "attempts" to 2
                )
            ),
            "apple" to arrayOf(
                mapOf(
                    "default" to false,
                    "source" to audioSources["yt-dlp"],
                    "convert" to { musicResponse: MusicApiResponse ->
                        if (musicResponse.isrc == null) {
                            querySong("${musicResponse.artist} ${musicResponse.title}", "youtube")?.get(0)
                        } else {
                            querySong(musicResponse.isrc, "youtube")?.get(0) ?:
                            querySong("${musicResponse.artist} ${musicResponse.title}", "youtube")?.get(0)
                        }
                    },
                    "attempts" to 2
                ),
                mapOf(
                    "default" to true,
                    "convert" to { musicResponse: MusicApiResponse ->
                        querySong("${musicResponse.artists} ${musicResponse.title}", "soundcloud")?.get(0)
                    }
                ),
                mapOf(
                    "default" to false,
                    "source" to audioSources["ytiz"],
                    "convert" to { musicResponse: MusicApiResponse ->
                        if (musicResponse.isrc == null) {
                            querySong("${musicResponse.artist} ${musicResponse.title}", "youtube")?.get(0)
                        } else {
                            querySong(musicResponse.isrc, "youtube")?.get(0) ?:
                            querySong("${musicResponse.artist} ${musicResponse.title}", "youtube")?.get(0)
                        }
                    },
                    "attempts" to 2
                )
            ),
            "youtube" to arrayOf(
                mapOf(
                    "default" to false,
                    "source" to audioSources["yt-dlp"],
                    "attempts" to 2
                ),
                mapOf(
                    "default" to true,
                    "convert" to { musicResponse: MusicApiResponse ->
                        querySong("${musicResponse.artists} ${musicResponse.title}", "soundcloud")?.get(0)
                    }
                ),
                mapOf(
                    "default" to false,
                    "source" to audioSources["ytiz"],
                    "attempts" to 2
                )
            ),
            "soundcloud" to arrayOf(
                mapOf(
                    "default" to true
                )
            )
        )

        init {
            println("\u001B[33m[Boater] \u001B[0mCreated main manager")
            AudioSourceManagers.registerRemoteSources(playerManager)
            AudioSourceManagers.registerLocalSource(playerManager)
            CoroutineScope(Dispatchers.IO).launch {
                while (true) {
                    refreshSpotifyToken()
                    delay(3_500_000)
                }
            }
        }

        internal fun fetchData(url: String,
                               method: String,
                               reqHeaders: Map<String, String>? = null,
                               body: String? = null,
                               plain: Boolean? = null,
                               bytes: Boolean? = null,
                               callback: (Response) -> Unit) {
            val requestBody = body?.toRequestBody()
            val request = Request.Builder()

            if (method == "GET" && requestBody == null) request.get() else if (method == "POST" && requestBody != null) request.post(requestBody)
            if (reqHeaders != null) request.headers(reqHeaders.toHeaders())
            request.url(url)

            try {
                okHttpClient.newCall(request.build()).enqueue(object : Callback {
                    override fun onFailure(call: Call, e: IOException) {
                        println(e)
                    }

                    override fun onResponse(call: Call, response: okhttp3.Response) {
                        if (!response.isSuccessful) {
                            response.close()
                            callback(Response(response.code))
                            return
                        }
                        val responseBody: Map<String, JsonElement> = if (plain == true) {
                            mapOf("plain" to json.encodeToJsonElement(response.body.string()))
                        } else if (bytes == true) {
                            mapOf("bytes" to json.encodeToJsonElement(Base64.getEncoder().encodeToString(response.body.bytes())))
                        } else {
                            Json.parseToJsonElement(response.body.string()).jsonObject
                        }

                        response.close()
                        callback(Response(response.code, response.headers.toMultimap(), responseBody))
                    }
                })
            } catch (e: InterruptedException) {
                Thread.currentThread().interrupt()
                println("fetch error")
            }
        }

        private fun refreshSpotifyToken() {
            val headers = mapOf(
                "Authorization" to "Basic ${Base64.getEncoder().encodeToString("${dotenv["SPOTIFY_CLIENT_ID"]}:${dotenv["SPOTIFY_CLIENT_SECRET"]}".toByteArray())}",
                "Content-Type" to "application/x-www-form-urlencoded"
            )
            fetchData("https://accounts.spotify.com/api/token", "POST", headers, "grant_type=client_credentials") {
                spotifyToken = it.body?.get("access_token")?.jsonPrimitive?.content as String
            }
        }

        fun newManager(instance: BotMusicManager) {
            println("\u001B[33m[Boater] \u001B[0mCreating new manager for ${instance.client.selfUser.name}")
            managers.add(instance)
        }

        fun getManager(client: JDA): BotMusicManager? {
            val index = managers.indexOfFirst { it.client == client }
            if (index == -1) return null
            return managers[index]
        }

        fun querySong(song: String, provider: String, amount: Int? = 1): List<MusicApiResponse>? {
            val results = CompletableFuture<List<MusicApiResponse>>()
            when (provider) {
                "spotify" -> {
                    val headers = mapOf(
                        "Authorization" to "Bearer $spotifyToken"
                    )
                    if (song.contains("open.spotify.com/track/")) {
                        val trackId = song.split('/').last()
                        fetchData("https://api.spotify.com/v1/tracks/${trackId}", "GET", headers) {
                            if (it.statusCode == 400) { results.complete(null); return@fetchData; }
                            val result = JsonArray(listOf(json.encodeToJsonElement(it.body))).map { element ->
                                MusicApiResponse.convertToMusicResponse(json.decodeFromJsonElement<SpotifyMusicResponse>(element))
                            }
                            results.complete(result)
                        }
                    } else {
                        fetchData("https://api.spotify.com/v1/search?type=track&q=${URLEncoder.encode(song, "UTF-8")}&limit=${amount}&market=US", "GET", headers) {
                            val result = (it.body?.get("tracks")?.jsonObject?.get("items")?.jsonArray)?.map { element ->
                                MusicApiResponse.convertToMusicResponse(json.decodeFromJsonElement<SpotifyMusicResponse>(element))
                            }
                            results.complete(result)
                        }
                    }
                }
                "apple" -> {
                    val headers = mapOf(
                        "Authorization" to "Bearer ${dotenv["APPLE_MUSIC_DEVELOPER_KEY"]}",
                        "Origin" to "https://music.apple.com"
                    )
                    if (song.contains("music.apple.com/")) {
                        val trackId = if (song.contains("?i=")) song.split('=').last() else song.split('/').last()
                        fetchData("https://api.music.apple.com/v1/catalog/us/songs/${trackId}", "GET", headers) {
                            val response = it.body?.get("data")?.jsonArray?.map { element ->
                                MusicApiResponse.convertToMusicResponse(json.decodeFromJsonElement<AppleMusicResponse>(element))
                            }
                            results.complete(response)
                        }
                    } else {
                        fetchData("https://api.music.apple.com/v1/catalog/us/search?types=songs,artists&limit=${amount}&term=${URLEncoder.encode(song, "UTF-8")}", "GET", headers) {
                            val response = (it.body?.get("results")?.jsonObject?.get("songs")?.jsonObject?.get("data") as JsonArray?)?.map { element ->
                                MusicApiResponse.convertToMusicResponse(json.decodeFromJsonElement<AppleMusicResponse>(element))
                            }
                            results.complete(response)
                        }
                    }
                }
                "youtube" -> {
                    if (song.contains("www.youtube.com/watch?v=")) {
                        val trackId = song.split("?v=").last().split('&').first()
                        fetchData("https://www.googleapis.com/youtube/v3/videos?id=$trackId&part=snippet,contentDetails&key=${dotenv["YOUTUBE_API_KEY"]}", "GET") {
                            val response = it.body?.get("items")?.jsonArray?.map { element ->
                                MusicApiResponse.convertToMusicResponse(json.decodeFromJsonElement<YoutubeMusicResponse>(element))
                            }
                            results.complete(response)
                        }
                    } else {
                        fetchData("https://www.googleapis.com/youtube/v3/search?q=${URLEncoder.encode(song, "UTF-8")}&type=video&part=snippet&key=${dotenv["YOUTUBE_API_KEY"]}", "GET") {
                            val response = it.body?.get("items")?.jsonArray?.map { element ->
                                MusicApiResponse.convertToMusicResponse(json.decodeFromJsonElement<YoutubeMusicResponse>(element))
                            }
                            results.complete(response)
                        }
                    }
                }
                "soundcloud" -> {
                    if (song.contains("api.soundcloud.com/")) {
                        val trackId = song.split("tracks/").last()
                        fetchData("https://api-v2.soundcloud.com/tracks/${trackId}?client_id=${dotenv["SOUNDCLOUD_CLIENT_ID"]}", "GET") {
                            val response = listOf(MusicApiResponse.convertToMusicResponse(json.decodeFromJsonElement<SoundCloudMusicResponse>(json.encodeToJsonElement(it.body))))
                            results.complete(response)
                        }
                    } else if (song.contains("soundcloud.com/")) {
                        fetchData("https://api-v2.soundcloud.com/resolve?url=${URLEncoder.encode(song, "UTF-8")}&client_id=${dotenv["SOUNDCLOUD_CLIENT_ID"]}", "GET") {
                            val response = listOf(MusicApiResponse.convertToMusicResponse(json.decodeFromJsonElement<SoundCloudMusicResponse>(json.encodeToJsonElement(it.body))))
                            results.complete(response)
                        }
                    } else {
                        fetchData("https://api-v2.soundcloud.com/search/tracks?q=${URLEncoder.encode(song, "UTF-8")}&client_id=${dotenv["SOUNDCLOUD_CLIENT_ID"]}", "GET") {
                            val response = it.body?.get("collection")?.jsonArray?.map { element ->
                                MusicApiResponse.convertToMusicResponse(json.decodeFromJsonElement<SoundCloudMusicResponse>(element))
                            }
                            results.complete(response)
                        }
                    }
                }
            }

            if (results.get() == null || results.get().isEmpty()) return null
            return results.get()
        }

        @Suppress("UNCHECKED_CAST")
        suspend fun createAudioTrack(musicResponse: MusicApiResponse, provider: String): AudioTrack? = withContext(Dispatchers.IO) {
            val loader = { audio: String, trackFuture: CompletableFuture<AudioTrack?> ->
                println(audio)
                playerManager.loadItem(audio, object : AudioLoadResultHandler { // Is cancelable
                    override fun trackLoaded(track: AudioTrack) {
                        println("song found")
                        trackFuture.complete(track)
                    }

                    override fun loadFailed(exception: FriendlyException?) {
                        println("error song failed")
                        trackFuture.complete(null)
                    }

                    override fun noMatches() {
                        println("no match")
                        trackFuture.complete(null)
                    }

                    override fun playlistLoaded(playlist: AudioPlaylist?) {}
                })
            }
            return@withContext suspendCoroutine { continuation ->
                var trackFuture: CompletableFuture<AudioTrack?> = CompletableFuture<AudioTrack?>()
                runBlocking {
                    for (service in providerSourceSelection[provider]!!) {
                        val convertedResponse = if (service["convert"] != null) {
                            (service["convert"] as (MusicApiResponse) -> MusicApiResponse?)(musicResponse)
                        } else {
                            musicResponse
                        }
                        if (convertedResponse == null) {
                            continue
                        }
                        if (service["default"] == true) {
                            println("default load")
                            loader(convertedResponse.url, trackFuture)
                            println("loaded")
                            continuation.resume(trackFuture.get())
                            return@runBlocking
                        }
                        val source = service["source"]!! as Map<String, *>
                        if (!(source["enabled"] as Boolean)) {
                            continue
                        }
                        for (i in 1..service["attempts"] as Int) {
                            trackFuture = CompletableFuture<AudioTrack?>()
                            val response = CompletableFuture<Map<String, JsonElement>>()
                            if (source["type"] == "internal") {
                                println("internal load")
                                loader((source["load"] as (MusicApiResponse) -> String)(convertedResponse), trackFuture)
                                println("loaded")
                                continuation.resume(trackFuture.get())
                                return@runBlocking
                            }
                            fetchData(
                                (source["url"] as (MusicApiResponse) -> String)(convertedResponse),
                                source["method"] as String,
                                source["headers"] as Map<String, String>?,
                                body = if (source.contains("body")) (source["body"] as (MusicApiResponse) -> String)(convertedResponse) else null,
                                plain = source["plainResponse"] as Boolean
                            ) {
                                println(source["name"])
                                if (it.body == null) {
                                    println("null body")
                                    response.complete(null)
                                } else {
                                    response.complete(it.body)
                                }
                            }
                            if (response.get() == null) {
                                trackFuture.complete(null)
                                continue
                            }
                            val audio = (source["processResponse"] as (Map<String, JsonElement>) -> String?)(response.get())
                            println(audio)
                            if (audio == null) {
                                trackFuture.complete(null)
                                continue
                            }
                            loader(audio, trackFuture)
                            if (trackFuture.await() != null) {
                                continuation.resume(trackFuture.get())
                                return@runBlocking
                            }
                        }
                    }
                    if (trackFuture.await() == null) continuation.resume(null) // returns fail if no service in list can provide audio track
                }
            }
        }

        fun connectVoice(guild: String, channel: String) {

        }

        fun connectVoice(guild: Guild?, channel: AudioChannelUnion?): AudioPlayer {
            val audioManager = guild?.audioManager
            val player = playerManager.createPlayer()
            audioManager?.sendingHandler = BoaterAudioSendHandler(player)
            audioManager?.openAudioConnection(channel)
            return player
        }

        fun addMessageButtons(message: Message, buttons: List<Button>, actionRow: BoaterActionRow) {
            message.editMessageComponents(ActionRow.of(buttons)).queue()
            messageButtonIdMap[message.id] = actionRow
        }

        fun removeMessageButtons(message: Message) {
            message.editMessageComponents().queue()
            messageButtonIdMap.remove(message.id)
        }

        fun createAudioFileString(artist: String, title: String): String {
            val fileExt = if (Config.botConfig.audioFileFormat == "vorbis") "ogg" else Config.botConfig.audioFileFormat
            return "./cache/songs/${artist.replace(Regex("[<>:\"/\\\\|?*]"), "")} - ${title.replace(Regex("[<>:\"/\\\\|?*]"), "")}.${fileExt}"
        }
    }

    data class Response(val statusCode: Int, val headers: Map<String, List<String>>? = null, val body: Map<String, JsonElement>? = null)
    class ObservableArrayList<T> : ArrayList<T>() {
        private val listeners = mutableListOf<(T) -> Unit>()

        fun addListener(listener: (T) -> Unit) {
            listeners.add(listener)
        }

        override fun add(element: T): Boolean {
            val added = super.add(element)
            if (added) listeners.forEach { it(element) }
            return added
        }
    }
}