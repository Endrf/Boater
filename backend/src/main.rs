#[macro_use] extern crate rocket;

use async_std::task;
use dotenv::dotenv;
use lazy_static::lazy_static;
use reqwest::Client;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::shield::{Shield, NoSniff, Frame};
use rocket::form::FromForm;
use rocket::fs::NamedFile;
use rocket::http::{Status, Header};
use rocket::request::Request;
use rocket::response::content::{RawHtml, RawJson};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::time::OffsetDateTime;
use rocket::{Response, State};
use rocket_ws::WebSocket;
use rocket_async_compression::{CachedCompression, Compression};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::string::ToString;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::env;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::sleep;

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open("../frontend/dist/index.html").await.ok()
}

#[get("/<path..>", rank = 2)]
async fn spa_index(path: PathBuf) -> Option<NamedFile> {
    let req_path = Path::new("../frontend/dist").join(&path);
    return if req_path.exists() {
        return Some(
            NamedFile::open(format!("../frontend/dist/{}", &path.display()))
                .await
                .ok()
                .unwrap(),
        );
    } else {
        NamedFile::open("../frontend/dist/index.html").await.ok()
    };
}

#[get("/.proxy/<path..>", rank = 3)]
async fn proxy_path_fallback(path: PathBuf) -> Option<NamedFile> {
    let req_path = Path::new("../frontend/dist").join(&path);
    return if req_path.exists() {
        return Some(
            NamedFile::open(format!("../frontend/dist/{}", &path.display()))
                .await
                .ok()
                .unwrap(),
        );
    } else {
        NamedFile::open("../frontend/dist/index.html").await.ok()
    };
}

#[get("/.proxy/ws/guild", rank = 1)]
async fn websocket_redirect(ws: WebSocket) -> rocket_ws::Stream!['static] {
    rocket_ws::Stream! { ws => 
        yield String::from("[reconnect] wss://boater-socket.endrf.com/guild").into()
    }
}

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

#[derive(Deserialize, FromForm)]
struct AuthCode {
    code: String,
}

#[get("/api/ping")]
async fn ping() -> Status {
    Status::Ok
}

#[post("/api/token", data = "<data>")]
async fn token(data: Json<AuthCode>) -> RawJson<String> {
    let code = &data.0.code;
    let client_id = env::var("DISCORD_CLIENT_ID").unwrap();
    let client_secret = env::var("DISCORD_CLIENT_SECRET").unwrap();
    let mut form_data = HashMap::new();
    form_data.insert("client_id", client_id.as_str());
    form_data.insert("client_secret", client_secret.as_str());
    form_data.insert("grant_type", "authorization_code");
    form_data.insert("code", &code);
    let response: Value = CLIENT.post("https://discord.com/api/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&form_data)
        .send().await.unwrap().json().await.unwrap();
    let token = serde_json::to_string(&response);
    RawJson(token.unwrap())
}

#[derive(Deserialize, FromForm)]
struct SongSearchParams {
    provider: String,
    data_type: String,
    search: String,
    offset: Option<u64>,
}

#[derive(Serialize, Clone)]
struct ServicePlaylist {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover: Option<String>,
    pub song_count: Option<u64>,
    pub provider: String,
}

#[derive(Serialize, Clone)]
struct ServiceSong {
    title: String,
    artists: String,
    id: String,
    url: String,
    cover: String,
    duration_ms: i64,
    release_date: String,
    provider: String,
}

#[get("/api/search?<params..>")]
async fn search(
    params: SongSearchParams,
    tokens: &State<AsyncMutex<AccessTokens>>,
) -> RawJson<String> {
    update_tokens(tokens).await;
    let tokens = tokens.lock().await;

    match params.data_type.as_str() {
        "songs" => RawJson(serde_json::to_string(
            &search_songs(params.provider, params.search, tokens.clone()).await
        ).unwrap()),
        "playlists" => RawJson(serde_json::to_string(
            &search_playlists(params.provider, params.search, tokens.clone()).await
        ).unwrap()),
        "playlist_songs" => RawJson(serde_json::to_string(
            &search_playlist_songs(params.provider, params.search, params.offset.unwrap(), tokens.clone()).await
        ).unwrap()),
        _ => RawJson(String::from("thing")),
    }
}

async fn search_songs(provider: String, search: String, tokens: AccessTokens) -> Vec<ServiceSong> {
    let results: AsyncMutex<Vec<ServiceSong>> = Default::default();
    if provider == *"spotify" {
        let mut results = results.lock().await;
        let response = CLIENT.get(format!("https://api.spotify.com/v1/search?type=track&q={search}&limit=8&market=US"))
            .bearer_auth(&tokens.spotify_token)
            .send().await.unwrap().json::<Value>().await.unwrap().get("tracks").unwrap().get("items")
            .unwrap().as_array().unwrap().clone();
        *results = response.iter().map(|song| ServiceSong {
            title: song.get("name").unwrap().as_str().unwrap().to_string(),
            artists: song.get("artists").unwrap().get(0).unwrap().get("name").unwrap().as_str().unwrap().to_string(),
            id: song.get("id").unwrap().as_str().unwrap().to_string(),
            url: song.get("external_urls").unwrap().get("spotify").unwrap().as_str().unwrap().to_string(),
            cover: song.get("album").unwrap().get("images").unwrap().get(0).unwrap().get("url").unwrap().as_str().unwrap().to_string(),
            duration_ms: song.get("duration_ms").unwrap().as_i64().unwrap(),
            release_date: song.get("album").unwrap().get("release_date").unwrap().as_str().unwrap().to_string(),
            provider: String::from("spotify"),
        }).collect();
    } else if provider == *"apple" {
        let mut results = results.lock().await;
        let response = CLIENT.get(format!("https://api.music.apple.com/v1/catalog/us/search?types=songs&limit=8&term={search}"))
            .header("Origin", "https://music.apple.com")
            .bearer_auth(&tokens.apple_token)
            .send().await.unwrap().json::<Value>().await.unwrap().get("results").unwrap().get("songs").unwrap().get("data")
            .unwrap().as_array().unwrap().clone();
        *results = response.iter().map(|song| {
            let attributes = song.get("attributes").unwrap();
            ServiceSong {
                title: attributes.get("name").unwrap().as_str().unwrap().to_string(),
                artists: attributes.get("artistName").unwrap().as_str().unwrap().to_string(),
                id: song.get("id").unwrap().as_str().unwrap().to_string(),
                url: attributes.get("url").unwrap().as_str().unwrap().to_string(),
                cover: attributes.get("artwork").unwrap().get("url").unwrap().as_str().unwrap().to_string().replace("{w}x{h}", "256x256"),
                duration_ms: attributes.get("durationInMillis").unwrap().as_i64().unwrap(),
                release_date: attributes.get("releaseDate").unwrap().as_str().unwrap().to_string(),
                provider: String::from("apple"),
            }
        }).collect();
    };
    let x = results.lock().await.clone();
    x
}

async fn search_playlists(
    provider: String,
    search: String,
    tokens: AccessTokens,
) -> Vec<ServicePlaylist> {
    let results: AsyncMutex<Vec<ServicePlaylist>> = Default::default();
    if provider == *"spotify" {
        let mut results = results.lock().await;
        let response = CLIENT.get(format!("https://api.spotify.com/v1/search?type=playlist&q={search}&limit=10&market=US"))
            .bearer_auth(&tokens.spotify_token)
            .send().await.unwrap().json::<Value>().await.unwrap().get("playlists").unwrap().get("items")
            .unwrap().as_array().unwrap().clone();
        *results = response.into_iter().filter(|playlist| !playlist.is_null()).map(|playlist| ServicePlaylist {
            title: playlist.get("name").unwrap().as_str().unwrap().to_string(),
            id: playlist.get("id").unwrap().as_str().unwrap().to_string(),
            artist: playlist.get("owner").unwrap().get("display_name").unwrap().as_str().unwrap().to_string(),
            cover: Some(playlist.get("images").unwrap().get(0).unwrap().get("url").unwrap().as_str().unwrap().to_string()),
            song_count: Some(playlist.get("tracks").unwrap().get("total").unwrap().as_u64().unwrap()),
            provider: String::from("spotify"),
        }).collect();
    } else if provider == *"apple" {
        let mut results = results.lock().await;
        let response: Vec<Value>;
        if search.starts_with("pl.") {
            response = CLIENT.get(format!("https://api.music.apple.com/v1/catalog/us/playlists/{search}"))
                .header("Origin", "https://music.apple.com")
                .bearer_auth(&tokens.apple_token)
                .send().await.unwrap().json::<Value>().await.unwrap().get("data")
                .unwrap().as_array().unwrap().clone()
        } else {
            response = CLIENT.get(format!("https://api.music.apple.com/v1/catalog/us/search?types=playlists&limit=10&term={search}"))
                .header("Origin", "https://music.apple.com")
                .bearer_auth(&tokens.apple_token)
                .send().await.unwrap().json::<Value>().await.unwrap().get("results").unwrap().get("playlists").unwrap().get("data")
                .unwrap().as_array().unwrap().clone();
        };
        *results = response.iter().map(|playlist| {
            let attributes = playlist.get("attributes").unwrap();
            ServicePlaylist {
                title: attributes.get("name").unwrap().as_str().unwrap().to_string(),
                id: playlist.get("id").unwrap().as_str().unwrap().to_string(),
                artist: attributes.get("curatorName").unwrap().as_str().unwrap().to_string(),
                cover: {
                    if attributes.get("artwork").is_some() {
                        Some(attributes.get("artwork").unwrap().get("url").unwrap().as_str().unwrap().to_string().replace("{w}x{h}", "256x256"))
                    } else {
                        None
                    }
                },
                song_count: None,
                provider: String::from("apple"),
            }
        }).collect();
    };
    let x = results.lock().await.clone();
    x
}

#[derive(Serialize, Default, Clone)]
struct ServicePlaylistSongs {
    next: Option<String>,
    songs: Vec<ServiceSong>,
}

async fn search_playlist_songs(
    provider: String,
    search: String,
    offset: u64,
    tokens: AccessTokens,
) -> ServicePlaylistSongs {
    let results: AsyncMutex<ServicePlaylistSongs> = Default::default();
    if provider == *"spotify" {
        let mut results = results.lock().await;
        let response = CLIENT.get(format!("https://api.spotify.com/v1/playlists/{search}/tracks?offset={offset}"))
            .bearer_auth(&tokens.spotify_token)
            .send().await.unwrap().json::<Value>().await.unwrap().clone();
        let songs = response.get("items").unwrap_or(&Value::Array(vec![])).as_array().unwrap().iter().map(|song| {
            let track = song.get("track").unwrap();
            ServiceSong {
                title: track.get("name").unwrap().as_str().unwrap().to_string(),
                artists: track.get("artists").unwrap().get(0).unwrap().get("name").unwrap().as_str().unwrap().to_string(),
                id: track.get("id").unwrap().as_str().unwrap().to_string(),
                url: track.get("external_urls").unwrap().get("spotify").unwrap().as_str().unwrap().to_string(),
                cover: track.get("album").unwrap().get("images").unwrap().get(0).unwrap().get("url").unwrap().as_str().unwrap().to_string(),
                duration_ms: track.get("duration_ms").unwrap().as_i64().unwrap(),
                release_date: track.get("album").unwrap().get("release_date").unwrap().as_str().unwrap().to_string(),
                provider: String::from("spotify"),
            }
        }).collect::<Vec<ServiceSong>>();
        *results = ServicePlaylistSongs {
            next: if response.get("next").unwrap().is_string() {
                Some(response.get("next").unwrap().as_str().unwrap().to_string())
            } else {
                None
            },
            songs,
        };
    } else if provider == *"apple" {
        let mut results = results.lock().await;
        let response = CLIENT.get(format!("https://api.music.apple.com/v1/catalog/us/playlists/{search}/tracks?offset={offset}"))
            .header("Origin", "https://music.apple.com")
            .bearer_auth(&tokens.apple_token)
            .send().await.unwrap().json::<Value>().await.unwrap().clone();
        let songs = response.get("data").unwrap_or(&Value::Array(vec![])).as_array().unwrap().iter().map(|song| {
            let attributes = song.get("attributes").unwrap();
            ServiceSong {
                title: attributes.get("name").unwrap().as_str().unwrap().to_string(),
                artists: attributes.get("artistName").unwrap().as_str().unwrap().to_string(),
                id: song.get("id").unwrap().as_str().unwrap().to_string(),
                url: attributes.get("url").unwrap().as_str().unwrap().to_string(),
                cover: attributes.get("artwork").unwrap().get("url").unwrap().as_str().unwrap().to_string().replace("{w}x{h}", "256x256"),
                duration_ms: attributes.get("durationInMillis").unwrap().as_i64().unwrap(),
                release_date: attributes.get("releaseDate").unwrap().as_str().unwrap().to_string(),
                provider: String::from("apple"),
            }
        }).collect::<Vec<ServiceSong>>();
        *results = ServicePlaylistSongs {
            next: if response.get("next").is_some() {
                Some(response.get("next").unwrap().as_str().unwrap().to_string())
            } else {
                None
            },
            songs,
        };
    }
    let x = results.lock().await.clone();
    x
}

#[derive(Deserialize, FromForm)]
struct LinkData {
    state: String,
}

#[post("/api/link", data = "<data>")]
async fn link(
    data: Json<LinkData>,
    request_data: &State<AsyncMutex<LoginRequests>>,
) -> RawJson<String> {
    request_data.lock().await.requests.insert(data.state.clone(), Arc::new(AsyncMutex::new(None)));
    let request = request_data.lock().await.requests.get(&data.state).unwrap().clone();

    // Remove request from state if resolve takes longer than 1 minute
    let request_clone =
        Arc::clone(request_data.lock().await.requests.get(&data.state).unwrap());
    task::spawn(async move {
        task::sleep(Duration::from_millis(60000)).await;
        *request_clone.lock().await = Some(String::from("error"));
    });

    while request.lock().await.is_none() {
        sleep(Duration::from_millis(500)).await
    }

    // Remove request from state
    let code = request_data.lock().await.requests.remove(&data.state).unwrap().lock().await.clone().unwrap();
    RawJson(format!(r#"{{"code": "{}"}}"#, code))
}

#[derive(Deserialize, FromForm)]
struct AuthorizeParams {
    code: Option<String>,
    state: String,
    error: Option<String>,
}

#[get("/api/authorize?<params..>")]
async fn authorize(
    params: AuthorizeParams,
    request_data: &State<AsyncMutex<LoginRequests>>,
) -> RawHtml<String> {
    if params.code.is_none() {
        *request_data.lock().await.requests.get(&params.state).unwrap().lock().await = Some(String::from("error"));
        RawHtml(String::from("Error"))
    } else {
        *request_data.lock().await.requests.get(&params.state).unwrap().lock().await = params.code;
        RawHtml(String::from("Ok"))
    }
}

async fn update_tokens(tokens: &State<AsyncMutex<AccessTokens>>) {
    let mut tokens = tokens.lock().await;
    if (OffsetDateTime::from(SystemTime::now()).unix_timestamp()
        - OffsetDateTime::from(tokens.spotify_expiration).unix_timestamp())
        > 3500
    {
        tokens.spotify_token = get_spotify_token().await;
        tokens.spotify_expiration = SystemTime::now();
    };
}

async fn get_spotify_token() -> String {
    let response: Value = CLIENT
        .post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .basic_auth(
            env::var("SPOTIFY_CLIENT_ID").unwrap(),
            env::var("SPOTIFY_CLIENT_SECRET").ok(),
        )
        .body("grant_type=client_credentials")
        .send().await.unwrap().json().await.unwrap();
    response.get("access_token").unwrap().as_str().unwrap().to_string()
}

#[derive(Clone)]
struct AccessTokens {
    spotify_token: String,
    spotify_expiration: SystemTime,
    apple_token: String,
}

struct LoginRequests {
    requests: HashMap<String, Arc<AsyncMutex<Option<String>>>>,
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let tokens = AccessTokens {
        spotify_token: get_spotify_token().await,
        spotify_expiration: SystemTime::now(),
        apple_token: env::var("APPLE_MUSIC_DEVELOPER_KEY").unwrap(),
    };

    let requests = LoginRequests {
        requests: HashMap::new(),
    };

    let shield = Shield::default().disable::<NoSniff>().disable::<Frame>();

    rocket::build()
        .attach(FrameOptionsFairing)
        .attach(shield)
        .attach(Compression::fairing())
        .attach(CachedCompression::path_suffix_fairing(vec![
            ".js".to_owned(),
            ".css".to_owned(),
            ".html".to_owned(),
            ".wasm".to_owned(),
        ]))
        .manage(AsyncMutex::new(tokens))
        .manage(AsyncMutex::new(requests))
        .mount(
            "/",
            routes![index, spa_index, proxy_path_fallback, websocket_redirect, ping, token, search, authorize, link],
        )
}

pub struct FrameOptionsFairing;

#[rocket::async_trait]
impl Fairing for FrameOptionsFairing {
    fn info(&self) -> Info {
        Info {
            name: "X-Frame-Options Header",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        res.remove_header("X-Frame-Options");
        if let Some(ext) = req.uri().path().split('.').last() {
            if ext == "wasm" {
                res.set_header(Header::new("Cache-Control", "no-transform"));
            }
        }
    }
}
