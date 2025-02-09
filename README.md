# Boater
### Boater is a Discord Music Bot with it's own [Discord Voice Channel Activity](https://discord.com/blog/server-activities-games-voice-watch-together)
### It supports a variety of different audio services and has some support for importing, saving, and playing entire playlists from your favorite audio service (currently working to improve support)
---
## Please read these warnings before using:
- The bot's websocket has no security (will fix this soon). Only give access to people you trust. There is also a setting in `config.toml` if you would like to disable the websocket and UI entirely.
- There are lots of missing or incomplete features (mostly on the UI side). **Stability is not guaranteed**.
- Streaming copyrighted content without permission can make you liable for copyright infringement. Previous Discord music bots have been taken down for this reason. I do not and likely will not host this bot for public use for this same reason since a higher population would likely increase liability. If you yourself choose to do this then you are doing so at your own risk.

## Please read this important technical information before using:
- This bot does not retrieve audio from Spotify or Apple Music. It only provides song, album, user, playlist data etc. from Spotify or Apple Music using their API and then searches for an audio stream that matches the song data using youtube (yt-dlp) or the lavaplayer soundcloud implementation.
- Using yt-dlp greatly increases the accuracy of correct song audio. To enable it, you must download an executable from [here](https://github.com/yt-dlp/yt-dlp/releases) that matches your system's architecture, place it in the `bot` folder and make sure to rename it to `yt-dlp` if it is not already, also set `useYtdlp = true` in `config.toml`, and set your `YOUTUBE_API_KEY` in `.env` ([See API Setup, Keys, and Secrets for more info on API keys](#api-setup-keys-and-secrets)).
- The bot and webserver can be run on separate systems. If you decide to do this, the bot and backend can be given their own `config.toml` and `.env` by putting these files in their respective directories. *This negates the main `config.toml` and/or `.env`
- The websocket is set up so that it listens on port `8081` and is routed through a cloudflare tunnel or reverse proxy so that the client can connect on the default `443` port. If you do not want to use a cloudflare tunnel or reverse proxy then you must run the websocket on port `443`.

## Installation
- Releases can be found [here](https://github.com/Endrf/Boater/releases)
- In-depth installation instructions will come soon. For now just refer to API setup and URL Mappings below.

## API Setup, Keys, and Secrets
> All API tokens are stored in the .env file
- `DISCORD_DEVELOPER_KEY` - Create a new application in the [Discord Developer Portal](https://discord.com/developers/applications) and retrieve the token from the bot section of your application. You may have to enable some of the privileged gateway intents for the bot to function properly.
- `DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET` - Found in the OAuth2 section of your app in the discord developer portal.
- `SPOTIFY_CLIENT_ID`, `SPOTIFY_CLIENT_SECRET` - Go to the [Spotify for Developers](https://developer.spotify.com/dashboard) dashboard, create a new app, retrieve the client ID and secret, add redirect URI (https://[YOUR **DISCORD** CLIENT ID].discordsays.com/api/authorize), and add authorized users for your app under user management so that they can link their accounts in the UI.
- `APPLE_MUSIC_DEVELOPER_KEY` - There are two ways to get this key
  - 1: Pay $99 to join the Apple Developer Program and make an app to get the key.
  - Or 2: Go to [music.apple.com](https://music.apple.com) and type `MusicKit.getInstance().developerToken` in the DevTools console and then copy the key. This key will only last a few months until it expires, in which you have to repeat the process to get a new key. <sub>There's probably a way to automate this though</sub>
- `YOUTUBE_API_KEY` - Go to [console.cloud.google.com](https://console.cloud.google.com), search for and enable YouTube Data API v3, create credentials, and copy the api key.
- `SoundCloud Client ID` - There are two ways to get this key
  - 1: Contact SoundCloud about making an app (their developer portal does not allow creating one yourself at the time I am writing this)
  - Or 2: Go to [soundcloud.com](https://soundcloud.com), open DevTools network section, search for requests containing `api` using filter, select the first occurrence and copy the id from `?client_id=[YOUR SOUNDCLOUD CLIENT ID]` in the url. This key will eventually expire but I am unsure how long it lasts.
 
## Discord Activity URL Mappings
> Insert these in the same order into the URL Mappings section of the Application in your Discord Developer Portal (this may not be necessary in the future since I may be able to proxy through the webserver)

| Prefix                         | Target                        |
| ------------------------------ | ----------------------------- |
| /i1/sndcdn/com                 | i1.sndcdn.com                 |
| /i1/ytimg/com                  | i1.ytimg.com                  |
| /js-cdn/music/apple/com        | js-cdn.music.apple.com        |
| /cdn/discordapp/com            | cdn.discordapp.com            |
| /accounts/spotify/com          | accounts.spotify.com          |
| /open/spotify/com              | open.spotify.com              |
| /api/spotify/com               | api.spotify.com               |
| /i/scdn/co                     | i.scdn.co                     |
| /image-cdn-fa/spotifycdn/com   | image-cdn-fa.spotifycdn.com   |
| /image-cdn-ak/spotifycdn/com   | image-cdn-ak.spotifycdn.com   |
| /seed-mix-image/spotifycdn/com | seed-mix-image.spotifycdn.com |
| /thisis-images/spotifycdn/com  | thisis-images.spotifycdn.com  |
| /pickasso/spotifycdn/com       | pickasso.spotifycdn.com       |
| /mosaic/scdn/co                | mosaic.scdn.co                |
| /misc/scdn/co                  | misc.scdn.co                  |
| /is1-ssl/mzstatic/com          | is1-ssl.mzstatic.com          |
| /i/ytimg/com                   | i.ytimg.com                   |
| /yt3/ggpht/com                 | yt3.ggpht.com                 |
| /ws                            | **[YOUR WEBSOCKET URL]**      |
| / (**ROOT**)                   | **[YOUR WEBSERVER URL]**      |

## Current Audio Service Support:
| Service     | Bot Support (Playing and Searching Songs) | UI Song Search         | UI Playlist Search     | UI Account Support     |
| ----------- | :---------------------------------------: | :--------------------: | :--------------------: | :--------------------: |
| Spotify     | <ul><li>[x] </li></ul>                    | <ul><li>[x] </li></ul> | <ul><li>[x] </li></ul> | <ul><li>[x] </li></ul> |
| Apple Music | <ul><li>[x] </li></ul>                    | <ul><li>[x] </li></ul> | <ul><li>[x] </li></ul> | <ul><li>[ ] </li></ul> |
| YouTube     | <ul><li>[x] </li></ul>                    | <ul><li>[ ] </li></ul> | <ul><li>[ ] </li></ul> | <ul><li>[ ] </li></ul> |
| SoundCloud  | <ul><li>[x] </li></ul>                    | <ul><li>[ ] </li></ul> | <ul><li>[ ] </li></ul> | <ul><li>[ ] </li></ul> |
| Tidal       | <ul><li>[ ] </li></ul>                    | <ul><li>[ ] </li></ul> | <ul><li>[ ] </li></ul> | <ul><li>[ ] </li></ul> |
