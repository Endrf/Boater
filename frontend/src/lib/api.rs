use crate::components::generic_playlist::{PlaylistData, SongData};
use crate::lib::util::{format_url, get_service_token_cookies};
use crate::lib::websocket::BoaterSong;
use futures::future::join;
use gloo::console::info;
use gloo::net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::window;
use yew::platform::spawn_local;
use yew::{Callback, UseStateHandle};

pub fn api_link_request(state: String, callback: Callback<String>) {
    spawn_local(async move {
        let response: HashMap<String, String> = Request::post(format_url("/api/link").as_str())
            .header("Content-Type", "application/json")
            .body(&format!(r#"{{"state": "{state}"}}"#))
            .unwrap().send().await.unwrap().json().await.unwrap();
        callback.emit(response.get("code").unwrap().to_string());
    });
}

// Match service and request final access token
pub async fn api_refresh_service_token(service: &str, refresh: String) -> HashMap<String, Value> {
    let url: &str;
    let mut body: String = String::new();
    match service {
        "spotify" => {
            url = "https://accounts.spotify.com/api/token";
            body = format!(
                "\
                client_id=e5757cf7de4b41df9f33abc374f15bf2\
                &grant_type=refresh_token\
                &refresh_token={refresh}"
            );
        }
        _ => url = "",
    }
    Request::post(&format_url(url))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .unwrap().send().await.unwrap().json().await.unwrap()
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct UserServiceData {
    pub name: String,
    pub avatar: String,
    pub playlist_count: u64,
    pub playlists: Option<Vec<ServicePlaylist>>,
    pub selected_playlist: Option<ServicePlaylist>,
    pub songs: Option<Vec<ServiceSong>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ServicePlaylist {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover: Option<String>,
    pub song_count: Option<u64>,
    pub provider: String,
}

impl PlaylistData for ServicePlaylist {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn title(&self) -> String {
        self.title.clone()
    }
    fn artist(&self) -> String {
        self.artist.clone()
    }
    fn cover(&self) -> Option<String> {
        self.cover.clone()
    }
    fn song_count(&self) -> Option<u64> {
        self.song_count
    }
    fn provider(&self) -> String {
        self.provider.clone()
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct ServiceSong {
    pub title: String,
    pub artists: String,
    pub id: String,
    pub url: String,
    pub cover: String,
    pub duration_ms: u64,
    pub release_date: String,
    pub provider: String,
}

impl SongData for ServiceSong {
    fn title(&self) -> String {
        self.title.clone()
    }
    fn artists(&self) -> String {
        self.artists.clone()
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn url(&self) -> String {
        self.url.clone()
    }
    fn cover(&self) -> String {
        self.cover.clone()
    }
    fn duration_ms(&self) -> u64 {
        self.duration_ms
    }
    fn release_date(&self) -> String {
        self.release_date.clone()
    }
    fn provider(&self) -> String {
        self.provider.clone()
    }
}

// Loads playlists of external service account
pub fn user_service_playlists_request(
    active_services: UseStateHandle<HashMap<&'static str, Option<String>>>,
    selected_service: &'static str,
    user_service_data: UseStateHandle<UserServiceData>,
    page: usize,
) {
    spawn_local(async move {
        let tokens = get_service_token_cookies().await;
        match selected_service {
            "spotify" => {
                let future1 = async {
                    Request::get(&format_url(format!("https://api.spotify.com/v1/me/playlists?limit=8&offset={}",(page - 1) * 8).as_str()))
                        .header("Authorization", &format!("Bearer {}", tokens.get("spotify").unwrap().clone().unwrap()))
                        .send().await.unwrap().json::<HashMap<String, Value>>().await.unwrap()
                };
                let future2 = async {
                    Request::get(&format_url("https://api.spotify.com/v1/me"))
                        .header("Authorization", &format!("Bearer {}", tokens.get("spotify").unwrap().clone().unwrap()))
                        .send().await.unwrap().json::<HashMap<String, Value>>().await.unwrap()
                };
                let (user_playlists, user_data) = join(future1, future2).await;
                let playlists_items = user_playlists.get("items").unwrap().as_array().unwrap();
                let formatted_playlists = playlists_items.iter().map(|playlist| ServicePlaylist {
                    id: playlist.get("id").unwrap().as_str().unwrap().to_string(),
                    title: playlist.get("name").unwrap().as_str().unwrap().to_string(),
                    artist: playlist.get("owner").unwrap().get("display_name").unwrap().as_str().unwrap_or(&*user_service_data.name).to_string(),
                    cover: {
                        if playlist.get("images").unwrap().as_array().is_none() {
                            None
                        } else {
                            Some(playlist.get("images").unwrap().as_array().unwrap().first().unwrap().as_object().unwrap().get("url")
                                .unwrap().as_str().unwrap().to_string()
                            )
                        }
                    },
                    song_count: Some(playlist.get("tracks").unwrap().get("total").unwrap().as_u64().unwrap()),
                    provider: selected_service.to_string(),
                }).collect::<Vec<ServicePlaylist>>();
                let data = UserServiceData {
                    name: user_data.get("display_name").unwrap().as_str().unwrap().to_string(),
                    avatar: user_data.get("images").unwrap().as_array().unwrap().first().unwrap().as_object().unwrap().get("url")
                        .unwrap().as_str().unwrap().to_string(),
                    playlist_count: user_playlists.get("total").unwrap().as_u64().unwrap(),
                    playlists: Some(formatted_playlists),
                    selected_playlist: None,
                    songs: None,
                };
                user_service_data.set(data)
            }
            _ => user_service_data.set(Default::default()),
        }
    })
}

// Loads songs for external service account playlists
pub fn user_service_playlist_songs_request(
    active_services: UseStateHandle<HashMap<&'static str, Option<String>>>,
    selected_service: &'static str,
    data_callback: Callback<Vec<ServiceSong>>,
    playlist: ServicePlaylist,
    mut songs: Vec<ServiceSong>,
    load_all: bool,
) {
    spawn_local(async move {
        let tokens = get_service_token_cookies().await;
        info!("load songs");
        match selected_service {
            "spotify" => {
                let mut playlist_songs = Request::get(&format_url(format!("https://api.spotify.com/v1/playlists/{}/tracks?limit=50&offset={}&fields=next,items(track(name,id,duration_ms,artists(name),external_urls(spotify),album(release_date,images(url))))", playlist.id, songs.len()).as_str()))
                    .header("Authorization", format!("Bearer {}", tokens.get("spotify").unwrap().clone().unwrap()).as_str())
                    .send().await.unwrap().json::<HashMap<String, Value>>().await.unwrap();
                let add_songs = {
                    move |playlist_songs: HashMap<String, Value>, songs: &mut Vec<ServiceSong>| {
                        let mut formatted = playlist_songs.get("items").unwrap().as_array().unwrap().iter().map(|song| ServiceSong {
                            title: song.get("track").unwrap().get("name").unwrap().as_str().unwrap().to_string(),
                            artists: song.get("track").unwrap().get("artists").unwrap().as_array().unwrap().first().unwrap().get("name")
                                .unwrap().as_str().unwrap().to_string(),
                            id: song.get("track").unwrap().get("id").unwrap().as_str().unwrap().to_string(),
                            url: song.get("track").unwrap().get("external_urls").unwrap().get("spotify").unwrap().as_str()
                                .unwrap().to_string(),
                            cover: song.get("track").unwrap().get("album").unwrap().get("images").unwrap().as_array()
                                .unwrap().first().unwrap().get("url").unwrap().as_str().unwrap().to_string(),
                            duration_ms: song.get("track").unwrap().get("duration_ms").unwrap().as_u64().unwrap(),
                            release_date: song.get("track").unwrap().get("album").unwrap().get("release_date")
                                .unwrap().as_str().unwrap().to_string(),
                            provider: String::from("spotify"),
                        }).collect::<Vec<ServiceSong>>();
                        songs.append(&mut formatted);
                    }
                };
                add_songs(playlist_songs.clone(), &mut songs);
                if load_all {
                    while playlist_songs.get("next").unwrap().is_string() {
                        playlist_songs = Request::get(&format_url(playlist_songs.get("next").unwrap().as_str().unwrap()))
                            .header("Authorization", format!("Bearer {}", tokens.get("spotify").unwrap().clone().unwrap()).as_str())
                            .send().await.unwrap().json::<HashMap<String, Value>>().await.unwrap();
                        add_songs(playlist_songs.clone(), &mut songs)
                    }
                }
                data_callback.emit(songs)
            }
            "apple" => {}
            _ => {}
        }
    })
}

pub fn playlist_songs_request(
    provider: String,
    data_callback: Callback<Vec<ServiceSong>>,
    song_count: UseStateHandle<u64>,
    playlist: ServicePlaylist,
    mut songs: Vec<ServiceSong>,
    load_all: bool,
) {
    spawn_local(async move {
        let hostname = window().expect_throw("no window").location().hostname().expect_throw("no hostname");
        let mut playlist_songs = Request::get(format!("https://{hostname}/.proxy/api/search?provider={provider}&data_type=playlist_songs&search={}&offset={}", playlist.id, songs.len()).as_str())
            .send().await.unwrap().json::<Value>().await.unwrap();
        let add_songs = {
            let song_count = song_count.clone();
            move |playlist_songs: Value, songs: &mut Vec<ServiceSong>| {
                let mut received_songs =
                    serde_json::from_value(playlist_songs.get("songs").unwrap().clone()).unwrap();
                songs.append(&mut received_songs);
                if playlist_songs.get("next").unwrap().is_null() {
                    song_count.set(songs.len() as u64)
                } else {
                    song_count.set((songs.len() + 1) as u64)
                }
            }
        };

        add_songs(playlist_songs.clone(), &mut songs);
        if load_all {
            while playlist_songs.get("next").unwrap().is_string() {
                playlist_songs = Request::get(format!("https://{hostname}/.proxy/api/search?provider={provider}&data_type=playlist_songs&search={}&offset={}", playlist.id, songs.len()).as_str())
                    .send().await.unwrap().json::<Value>().await.unwrap();
                add_songs(playlist_songs.clone(), &mut songs)
            }
        }
        data_callback.emit(songs)
    })
}
