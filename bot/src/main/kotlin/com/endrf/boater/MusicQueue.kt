package com.endrf.boater

import com.endrf.boater.buttons.NowPlayingActionRows
import com.endrf.boater.config.Config
import com.endrf.boater.embeds.ErrorEmbed
import com.endrf.boater.embeds.LoadingEmbed
import com.endrf.boater.embeds.NowPlayingEmbed
import com.endrf.boater.embeds.TrackAddEmbed
import com.sedmelluq.discord.lavaplayer.player.AudioPlayer
import com.sedmelluq.discord.lavaplayer.player.event.AudioEventAdapter
import com.sedmelluq.discord.lavaplayer.track.AudioTrack
import com.sedmelluq.discord.lavaplayer.track.AudioTrackEndReason
import com.squareup.moshi.adapter
import kotlinx.coroutines.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import net.dv8tion.jda.api.entities.Guild
import net.dv8tion.jda.api.entities.Message
import net.dv8tion.jda.api.entities.channel.unions.AudioChannelUnion
import java.util.concurrent.CompletableFuture
import kotlin.io.path.*

@OptIn(ExperimentalStdlibApi::class)
class MusicQueue(
    var song: PlaylistContainer,
    val guild: Guild,
    private val channel: AudioChannelUnion?
) {
    private val playerListener: AudioEventAdapter = object : AudioEventAdapter() {
        override fun onTrackEnd(player: AudioPlayer?, track: AudioTrack?, endReason: AudioTrackEndReason?) {
            if (!Config.botConfig.cacheSongs && track != null && track.identifier.startsWith("./cache")) {
                Thread.sleep(100)
                Path(track.identifier).deleteIfExists()
            }
            if (endReason == AudioTrackEndReason.FINISHED) {
                forcePlayFromQueue(inPlaylist = true)
            }
        }
    }
    private var currentTrack: AudioTrack? = null // change dis
    internal val songs = ArrayList<PlaylistContainer>()
    lateinit var player: AudioPlayer
    private val queueListeners = mutableListOf<(QueueUpdateResponse) -> Unit>()
    fun isPlayerInit() = this::player.isInitialized
    var loop = false
    var cancelCurrent: Boolean = false

    suspend fun initialize(): Boolean? {
        play(song) ?: return null

        CoroutineScope(Dispatchers.IO).launch {
            while (currentTrack != null) {
                runBlocking {
                    if (isPlayerInit() && player.playingTrack?.position != null && !player.isPaused ) {
                        queueListeners.forEach { it(QueueUpdateResponse(position = player.playingTrack.position)) }
                        delay(1_000)
                    }
                    if (currentTrack != null) delay(10)
                }
            }
        }

        return true
    }

    fun forcePlayFromQueue(index: Int = -1, inPlaylist: Boolean = false) {
        if (songs.isEmpty() && (song.songs.size == song.position + 1 || (index == -1 && !inPlaylist))) {
            selfDestruct()
            return
        }
        CoroutineScope(Dispatchers.IO).launch {
            if (index != -1 && inPlaylist) {
                song.position = index
                play(song)
            } else if (song.songs.size > song.position + 1 && inPlaylist) {
                song.position += 1
                play(song)
            } else if (index != -1) play(songs.removeAt(index))
            else play(songs.removeAt(0))
        }
    }

    private suspend fun play(newSong: PlaylistContainer) = withContext(Dispatchers.IO) {
        val message = CompletableFuture<Message?>()
        val loadingEmbed = CompletableFuture<LoadingEmbed>()

        launch {
            loadingEmbed.complete(LoadingEmbed(newSong) { message.complete(it) })
        }

        val track = BotMusicManager.createAudioTrack(newSong.songs[newSong.position], newSong.songs[newSong.position].provider)
        println("track: $track")
        loadingEmbed.get().finish()

        if (track == null) {
            message.get()?.editMessageEmbeds(ErrorEmbed("Could not find an audio stream for that track\n*If the problem persists, try a different search provider*").build())?.queue()
            if (isPlayerInit() && player.playingTrack == null) forcePlayFromQueue(inPlaylist = true)
            return@withContext null
        } else if (cancelCurrent) {
            cancelCurrent = false
            message.get()?.delete()?.queue()
            return@withContext null
        }

        if (!guild.audioManager.isConnected) {
            this@MusicQueue.player = BotMusicManager.connectVoice(guild, channel)
            player.addListener(playerListener)
        }

        if (message.get() != null) {
            // ! Issue removing buttons after playing chat request then UI request
            currentTrack?.let { (currentTrack?.userData as Message?)?.let { message -> BotMusicManager.removeMessageButtons(message) } }
            track.userData = message.get()
        }

        currentTrack = track
        // If newSong is playlist, re-assign the song data in the current position to get artist avatar and other song data
        if (newSong.playlistData != null) {
            newSong.songs[newSong.position] = BotMusicManager.querySong(newSong.songs[newSong.position].url, newSong.songs[newSong.position].provider)!![0]
        }
        song = newSong // Set instances song only when song resource is confirmed and not canceled

        player.playTrack(track)
        player.isPaused = false
        message.get()?.editMessageEmbeds(NowPlayingEmbed(newSong, this@MusicQueue).build())?.queue()
        message.get()?.let { BotMusicManager.addMessageButtons(it, NowPlayingActionRows.controllerMain(this@MusicQueue), NowPlayingActionRows) }
        queueListeners.forEach { it(QueueUpdateResponse(
            song = song.songs[song.position],
            playlist = Json.parseToJsonElement(BotMusicManager.moshi.adapter<ServicePlaylist>().toJson(song.playlistData)),
            playlistPosition = song.position,
            playlistSongs = song.songs,
            queue = InteractionProperties(queueData = songs).formattedQueueData,
            interaction = InteractionProperties(song.interaction).userData,
            isPaused = player.isPaused,
            position = player.playingTrack.position
        )) }
    }

    fun addSong(newSong: PlaylistContainer) {
        songs.add(newSong)
        newSong.interaction.interaction?.replyEmbeds(TrackAddEmbed(newSong, this).build())?.queue()
        newSong.interaction.message = newSong.interaction.interaction?.hook?.retrieveOriginal()?.complete()
        newSong.interaction.interaction = null
        queueListeners.forEach { it(QueueUpdateResponse(queue = InteractionProperties(queueData = songs).formattedQueueData)) }
    }

    fun removeSong(songIndex: Int): PlaylistContainer {
        val removedSong = songs.removeAt(songIndex)
        queueListeners.forEach { it(QueueUpdateResponse(queue = InteractionProperties(queueData = songs).formattedQueueData)) }
        return removedSong
    }

    fun togglePausePlay() {
        // Possibly change this function to update embed controller buttons instead of the embed itself updating the buttons to fix sync issue with webUI
        if(!player.isPaused) player.isPaused = true
        else if(player.isPaused) player.isPaused = false
        queueListeners.forEach { it(QueueUpdateResponse(isPaused = player.isPaused)) }
    }

    fun setVolume(level: Int) {
        player.volume = level
    }

    fun addListener(listener: (QueueUpdateResponse) -> Unit) {
        queueListeners.add(listener)
    }

    fun selfDestruct() {
        (currentTrack?.userData as Message?)?.let { BotMusicManager.removeMessageButtons(it) }
        song.interaction.message?.reply("No more songs in queue. Disconnecting from voice channel.")?.queue()

        guild.audioManager.closeAudioConnection()
        player.destroy()
        guild.audioManager.sendingHandler = null
        currentTrack = null
        BotMusicManager.getManager(guild.jda)?.removeQueue(this@MusicQueue)
        BotMusicManager.playerManager.executor.execute { player.removeListener(playerListener) }
        queueListeners.forEach { it(QueueUpdateResponse()) }
    }

    fun initialData(): QueueUpdateResponse {
        return QueueUpdateResponse(
            song = song.songs[song.position],
            playlist = Json.parseToJsonElement(BotMusicManager.moshi.adapter<ServicePlaylist>().toJson(song.playlistData)),
            playlistPosition = song.position,
            playlistSongs = song.songs,
            queue = InteractionProperties(queueData = songs).formattedQueueData,
            interaction = InteractionProperties(song.interaction).userData,
            isPaused = player.isPaused,
            position = player.playingTrack?.position
        )
    }

    @Suppress("PLUGIN_IS_NOT_ENABLED")
    @Serializable
    data class QueueUpdateResponse(
        val song: MusicApiResponse? = null,
        val playlist: JsonElement? = null,
        val playlistPosition: Int? = null,
        val playlistSongs: ArrayList<MusicApiResponse>? = null,
        val queue: List<QueueUpdateResponse>? = null,
        val interaction: Map<String, String?>? = null,
        val isPaused: Boolean? = null,
        val position: Long? = null
    )

    data class InteractionProperties(
        private val interactionData: UserInteraction? = null,
        private val queueData: ArrayList<PlaylistContainer>? = null
    ) {
        val userData: Map<String, String?>?
            get() {
                val user = interactionData ?: return null
                return mapOf(
                    "name" to user.userName,
                    "avatarUrl" to user.avatarUrl
                )
        }

        val formattedQueueData: List<QueueUpdateResponse>?
            get() {
                return queueData?.map { QueueUpdateResponse(
                    song = it.songs[it.position],
                    interaction = InteractionProperties(it.interaction).userData,
                    playlist = Json.parseToJsonElement(BotMusicManager.moshi.adapter<ServicePlaylist>().toJson(it.playlistData))
                ) }
            }
    }

}