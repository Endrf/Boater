use crate::components::generic_playlist::{GenericSong, GenericPlaylist, PlaylistData, SongData};
use crate::components::modal::{ModalAttributes, ModalColor, ModalType};
use crate::lib::api::{ServicePlaylist, ServiceSong};
use crate::lib::util::{format_url, service_icon_html};
use crate::{getDisplayName, getGuildId, getUserAvatar, getUserId, getUserName};
use serde::{Deserialize, Serialize};
use serde_json::{to_string, to_value, Value};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use gloo::console::info;
use web_sys::js_sys::WebAssembly::Global;
use web_sys::{WebSocket, HtmlInputElement};
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew::html::IntoPropValue;
use yew::html::onclick::Event as ClickEvent;
use yew::html::onchange::Event as ChangeEvent;
use yew::{html, Callback, UseStateHandle};
use crate::routes::guild::DefaultProps;

#[derive(Default, Serialize, Clone)]
pub struct WebSocketPayload {
    pub action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataObject: Option<Value>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub user: Option<&'static str>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub provider: Option<&'static str>
}

#[derive(Serialize)]
struct BoaterPlayRequest {
    url: String,
    provider: String,
    user: BoaterInteraction,
}

#[derive(Serialize)]
struct BoaterPlaylistRequest {
    playlist: ServicePlaylist,
    songs: Vec<ServiceSong>,
    user: BoaterInteraction
}

#[derive(Serialize)]
struct BoaterInteraction {
    id: String,
    userName: String,
    avatarUrl: String,
    guildId: String,
}

pub fn add_queue_song(
    ws: WebSocket,
    modal_attributes_state: UseStateHandle<ModalAttributes>,
    song: ServiceSong,
) {
    modal_attributes_state.set(ModalAttributes {
        title: String::from("Add Confirmation"),
        content: html! {
            <p>
                {"Are you sure you want to add"}
                <br /><strong>
                    {song.title}{" by "}{song.artists}
                </strong><br />
                {"to the server's queue?"}
            </p>
        },
        color: ModalColor::Primary,
        confirm: {
            let modal_attributes_state = modal_attributes_state.clone();
            Callback::from(move |_| {
                ws.clone()
                    .send_with_str(
                        to_string(&WebSocketPayload {
                            action: "play",
                            dataObject: Some(
                                to_value(BoaterPlayRequest {
                                    url: song.url.clone(),
                                    provider: song.provider.clone(),
                                    user: BoaterInteraction {
                                        id: getUserId().unwrap_or_default(),
                                        userName: getUserName(),
                                        avatarUrl: getUserAvatar(),
                                        guildId: getGuildId(),
                                    },
                                })
                                .unwrap(),
                            ),
                            ..Default::default()
                        })
                        .unwrap()
                        .as_str(),
                    )
                    .unwrap();
                modal_attributes_state.set(ModalAttributes {
                    open: false,
                    ..Default::default()
                });
            })
        },
        ..Default::default()
    });
}

pub fn add_queue_playlist(
    ws: WebSocket,
    modal_attributes_state: UseStateHandle<ModalAttributes>,
    playlist: ServicePlaylist,
    load_all_fn: Callback<(ServicePlaylist, Callback<Vec<ServiceSong>>)>
) {
    modal_attributes_state.clone().set(ModalAttributes {
        title: String::from("Add Confirmation"),
        content: html! {
            <p>
                {"Are you sure you want to add"}
                <br /><strong>
                {playlist.title.clone()}{" by "}{playlist.artist.clone()}
                </strong><br />
                {"to the server's queue?"}
            </p>
        },
        color: ModalColor::Primary,
        confirm: {
            Callback::from(move |_| {
                let ws = ws.clone();
                let modal_attributes_state = modal_attributes_state.clone();
                let playlist = playlist.clone();
                load_all_fn.emit((playlist.clone(), Callback::from(move |songs: Vec<ServiceSong>| {
                    ws.send_with_str(to_string(&WebSocketPayload {
                        action: "playPlaylist",
                        dataObject: Some(
                            to_value(BoaterPlaylistRequest {
                                playlist: playlist.clone(),
                                songs,
                                user: BoaterInteraction {
                                    id: getUserId().unwrap_or_default(),
                                    userName: getUserName(),
                                    avatarUrl: getUserAvatar(),
                                    guildId: getGuildId()
                                }
                            }).unwrap()
                        ),
                        ..Default::default()
                    }).unwrap().as_str()).unwrap();
                    modal_attributes_state.set(ModalAttributes {open: false, ..Default::default()});
                })));
            })
        },
        ..Default::default()
    });
}

#[derive(Serialize)]
struct BoaterPlaylistPayload {
    playlist: ServicePlaylist,
    songs: Vec<ServiceSong>,
    user: BoaterUser,
}

#[derive(Serialize, Deserialize)]
pub struct BoaterUser {
    pub id: String,
    pub display_name: String,
    pub avatar: String,
}

#[derive(Deserialize, Default, Clone, PartialEq)]
pub struct BoaterUserPlaylist {
    pub id: String,
    pub title: String,
    pub owner: String,
    pub artist: String,
    pub cover: Option<String>,
    pub song_count: u64,
    pub provider: String,
}

impl PlaylistData for BoaterUserPlaylist {
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
        Some(self.song_count)
    }
    fn provider(&self) -> String {
        self.provider.clone()
    }
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct BoaterSong {
    pub title: String,
    pub artists: String,
    pub id: String,
    pub url: String,
    pub cover: String,
    pub duration_ms: u64,
    pub release_date: String,
    pub provider: String,
}

impl SongData for BoaterSong {
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

impl Into<ServiceSong> for BoaterSong {
    fn into(self) -> ServiceSong {
        ServiceSong {
            title: self.title,
            artists: self.artists,
            id: self.id,
            url: self.url,
            cover: self.cover,
            duration_ms: self.duration_ms,
            release_date: self.release_date,
            provider: self.provider,
        }
    }
}

pub fn save_playlist(
    ws: WebSocket,
    modal_attributes_state: UseStateHandle<ModalAttributes>,
    playlist: ServicePlaylist,
    conflict_callback: UseStateHandle<Callback<Value>>,
    load_all_fn: Callback<(ServicePlaylist, Callback<Vec<ServiceSong>>)>,
) {
    ws.send_with_str(
        // Check if playlist exists in db
        to_string(&WebSocketPayload {
            action: "getPlaylistExists",
            dataObject: Some(
                to_value(HashMap::from([
                    ("id", playlist.id.as_str()),
                    ("owner", getUserId().unwrap_or_default().as_str()),
                ]))
                .unwrap(),
            ),
            ..Default::default()
        })
        .unwrap()
        .as_str(),
    )
    .unwrap();
    modal_attributes_state.set(ModalAttributes {
        title: String::from("Save Playlist Confirmation"),
        content: {
            html! {
                <div class="flex justify-center p-[1vw] font-semibold text-[1.2vw]">
                    {"Loading..."}
                </div>
            }
        },
        modal_type: ModalType::Loading,
        ..Default::default()
    });
    // Callback that is called when websocket returns db conflict data
    conflict_callback.set({
        let ws = ws.clone();
        let load_all_fn = load_all_fn.clone();
        Callback::from(move |existing_playlist: Value| {
            let confirm = {
                let ws = ws.clone();
                let modal_attributes_state = modal_attributes_state.clone();
                let playlist = playlist.clone();
                let load_all_fn = load_all_fn.clone();
                Callback::from(move |_| { // Callback called when confirm button clicked
                    let ws = ws.clone();
                    let modal_attributes_state = modal_attributes_state.clone();
                    let playlist = playlist.clone();
                    modal_attributes_state.set(ModalAttributes {
                        title: String::from("Save Playlist"),
                        content: html! {<>{"Downloading Song Data..."}</>},
                        modal_type: ModalType::Loading,
                        ..Default::default()
                    });
                    load_all_fn.emit((playlist.clone(), Callback::from(move |songs: Vec<ServiceSong>| {
                        let mut playlist = playlist.clone();
                        playlist.song_count = Some(songs.len() as u64);
                        ws.clone().send_with_str(to_string(&WebSocketPayload {
                            action: "addUserPlaylist",
                            dataObject: Some(to_value(BoaterPlaylistPayload {
                                playlist,
                                songs,
                                user: get_client_user_obj()
                            }).unwrap()),
                            ..Default::default()
                        }).unwrap().as_str()).unwrap();
                        modal_attributes_state.set(ModalAttributes {
                            modal_type: ModalType::Notification,
                            color: ModalColor::Success,
                            title: String::from("Playlist Saved"),
                            content: html! {<>{"The playlist has been saved."}</>},
                            confirm: {
                                let modal_attributes_state = modal_attributes_state.clone();
                                Callback::from(move |_| modal_attributes_state.set(ModalAttributes { open: false, ..Default::default() }))
                            },
                            ..Default::default()
                        });
                    })));
                })
            };
            if existing_playlist.is_null() { // If playlist does not exist, else if playlist exists
                modal_attributes_state.set(ModalAttributes {
                    title: String::from("Save Playlist Confirmation"),
                    content: {
                        let playlist = playlist.clone();
                        html! {
                            <div class="flex flex-col gap-[0.5vw]">
                                {"Are you sure you want to save this playlist?"}
                                <div class="w-full flex flex-row">
                                    <div class="relative h-full aspect-square">
                                        <div class="absolute top-[0.2vw] left-[0.2vw] py-[0.2vw] px-[0.4vw] text-[0.8vw] rounded-md bg-neutral-800/70 backdrop-blur z-[1]">
                                            {service_icon_html(playlist.provider)}
                                        </div>
                                        <img class="rounded-l" width="128" height="128" src={format_url(playlist.cover.unwrap_or_default().as_str())} />
                                    </div>
                                    <div class="w-full p-[0.5vw] text-left text-[1.2vw] flex flex-col bg-neutral-900 rounded-r">
                                        <span class="font-semibold">{playlist.title}</span>
                                        <span class="text-neutral-400 text-[1vw]">{"By "}{playlist.artist}</span>
                                    </div>
                                </div>
                            </div>
                        }
                    },
                    color: ModalColor::Primary,
                    confirm,
                    ..Default::default()
                })
            } else {
                modal_attributes_state.set(ModalAttributes {
                    title: String::from("Playlist Overwrite Confirmation"),
                    content: html! {
                        <div class="flex justify-center p-[1vw] text-[1.2vw]">
                            {"You've already saved this playlist. Do you want to overwrite it?"}
                            <br />
                            {existing_playlist.get("title").unwrap().as_str().unwrap().to_string()}
                        </div>
                    },
                    color: ModalColor::Warning,
                    confirm,
                    ..Default::default()
                })
            }
        })
    })
}

pub fn add_song_to_playlist(
    default_props: DefaultProps,
    song: ServiceSong,
) {
    let ws = (*default_props.ws).clone();
    let modal_attributes_state = default_props.modal_attributes;
    let conflict_callback = default_props.db_response_callback;
    let open_playlist = default_props.selected_boater_playlist;
    let selected_playlist = Rc::new(RefCell::new(BoaterUserPlaylist::default()));
    let modal_content = {
        let selected_playlist = selected_playlist.clone();
        Box::new(move |user_playlists: Option<Vec<BoaterUserPlaylist>>| {
            html! {
                <div>
                    <select onchange={let user_playlists = user_playlists.clone(); let selected_playlist_state = selected_playlist.clone(); Callback::from(move |event: ChangeEvent| *selected_playlist_state.borrow_mut() = user_playlists.clone().unwrap().into_iter().find(|x| x.id == event.target().unwrap().unchecked_into::<HtmlInputElement>().value()).unwrap())}>
                        if user_playlists.is_some() {
                            <option value="" disabled=true selected=true>{"Select a playlist"}</option>
                            <option value="">{"Create new playlist"}</option> // change to button
                                                                              // on side of select
                            { for user_playlists.unwrap().iter().map(|playlist| {
                                html! {
                                    <option value={playlist.id.clone()}>{playlist.title.clone()}</option>
                                }
                            }) }
                        } else {
                            <option value="" disabled=true selected=true>{"Loading user playlists..."}</option>
                        }
                    </select>
                </div>
            }
        })
    };
    // Loading Modal
    modal_attributes_state.set(ModalAttributes {
        title: String::from("Add Song to Playlist"),
        content: modal_content(None),
        ..Default::default()
    });
    conflict_callback.set({
        let props = (ws.clone(), modal_attributes_state.clone(), conflict_callback.clone(), song.clone(), open_playlist.clone(), selected_playlist.clone());
        Callback::from(move |response: Value| {
            let mut user_playlists = if let Some(db_user_playlists) = response.get("userPlaylists") {
                serde_json::from_value::<Vec<BoaterUserPlaylist>>(db_user_playlists.clone()).unwrap()
            } else { Vec::new() };
            user_playlists.reverse();
            let props = props.clone();
            // Main "Add" Modal
            modal_attributes_state.set(ModalAttributes {
                title: String::from("Add Song to Playlist"),
                content: modal_content(Some(user_playlists)),
                confirm: Callback::from(move |e| {
                    let props = props.clone();
                    let modal_attributes_state = props.1.clone();
                    let selected_playlist = props.5.clone();
                    let add_songs = {
                        let ws = props.0.clone();
                        let modal_attributes_state = props.1.clone();
                        let conflict_callback = props.2.clone();
                        let song = props.3.clone();
                        let selected_playlist = props.5.clone();
                        Callback::from(move |_| {
                            // Sets callback for add respond event
                            conflict_callback.set({
                                let ws = props.0.clone();
                                let open_playlist = props.4.clone();
                                let selected_playlist = selected_playlist.clone();
                                Callback::from(move |response2: Value| {
                                    let playlist_id = if let Some(id) = response2.get("newId") {
                                        id.as_str().unwrap().to_string()
                                    } else {
                                        selected_playlist.borrow().id.clone()
                                    };
                                    if (*open_playlist).clone().unwrap_or_default() == selected_playlist.borrow().id {
                                        get_boater_playlist_songs(ws.clone(), playlist_id.clone(), getUserId().unwrap());
                                        open_playlist.set(Some(playlist_id));
                                        get_boater_user_playlists(ws.clone(), getUserId().unwrap())
                                    }
                                })
                            });
                            ws.clone().send_with_str(
                                to_string(&WebSocketPayload {
                                    action: "addPlaylistSongs",
                                    dataObject: Some(to_value(HashMap::from([
                                        ("id", Value::from(selected_playlist.borrow().id.clone())),
                                        ("songs", to_value(vec![song.clone()]).unwrap()),
                                        ("user", to_value(get_client_user_obj()).unwrap()),
                                        ("position", Value::from(selected_playlist.borrow().song_count.clone())),
                                        ("convert", Value::from(selected_playlist.borrow().provider != "boater"))
                                    ])).unwrap()),
                                    ..Default::default()
                                }).unwrap().as_str()
                            ).unwrap_or_default();
                            // Success Modal
                            modal_attributes_state.set(ModalAttributes {
                                title: String::from("Song Added"),
                                content: html! {
                                    <>{"The song was added to the playlist"}</>
                                },
                                modal_type: ModalType::Notification,
                                color: ModalColor::Success,
                                confirm: {
                                    let modal_attributes_state = modal_attributes_state.clone();
                                    Callback::from(move |_| { 
                                        modal_attributes_state.set(ModalAttributes {open: false, ..Default::default()});
                                    })
                                },
                                ..Default::default()
                            });
                            //get_boater_user_playlists(ws.clone(), getUserId().unwrap())
                        })
                    };
                    // Extra modal if not boater playlist
                    if selected_playlist.borrow().provider == String::from("boater") {
                        add_songs.emit(e)
                    } else {
                        modal_attributes_state.set(ModalAttributes {
                            title: String::from("Add Song to Playlist"),
                            content: html! {
                                <div class="flex flex-col">
                                    <p>{"This playlist is linked to "}<strong>{selected_playlist.borrow().provider.clone()}</strong>
                                    <br />
                                    {"Do you want to convert it to a custom Boater playlist?"}
                                    <br />
                                    {"*Cancelling will not add the song to the playlist"}</p>
                                </div>
                            },
                            modal_type: ModalType::Confirmation,
                            color: ModalColor::Warning,
                            confirm: add_songs.clone(),
                            ..Default::default()
                        })
                    }
                }),
                ..Default::default()
            })
        })
    });
    // Refreshes / loads user playlists for modal select
    get_boater_user_playlists(ws.clone(), getUserId().unwrap());
}

pub fn remove_song_from_playlist(
    default_props: DefaultProps,
    playlist: GenericPlaylist,
    song: ServiceSong,
    position: usize
) {
    let ws = (*default_props.ws).clone();
    let modal_attributes_state = default_props.modal_attributes;
    let conflict_callback = default_props.db_response_callback;
    let open_playlist = default_props.selected_boater_playlist;
    
    let remove_song = {
        let modal_attributes_state = modal_attributes_state.clone();
        let playlist = playlist.clone();
        Callback::from(move |_| {
            conflict_callback.set({
                let ws = ws.clone();
                let open_playlist = open_playlist.clone();
                let playlist = playlist.clone();
                Callback::from(move |response2: Value| {
                    let playlist_id = if let Some(id) = response2.get("newId") {
                        id.as_str().unwrap().to_string()
                    } else {
                        playlist.playlist().id()
                    };
                    if (*open_playlist).clone().unwrap_or_default() == playlist.playlist().id() {
                        get_boater_playlist_songs(ws.clone(), playlist_id.clone(), getUserId().unwrap());
                        open_playlist.set(Some(playlist_id));
                        get_boater_user_playlists(ws.clone(), getUserId().unwrap())
                    }
                })
            });
            ws.send_with_str(to_string(&WebSocketPayload {
                action: "removePlaylistSong",
                dataObject: Some(to_value(HashMap::from([
                    ("id", Value::from(playlist.playlist().id())),
                    ("position", Value::from(position)),
                    ("user", to_value(get_client_user_obj()).unwrap()),
                    ("convert", Value::from(playlist.playlist().provider() != "boater"))
                ])).unwrap()),
                ..Default::default()
            }).unwrap().as_str()).unwrap();
            // Success Modal
            modal_attributes_state.set(ModalAttributes {
                title: String::from("Song Removed"),
                content: html! {
                    <>{"The song was removed from the playlist"}</>
                },
                modal_type: ModalType::Notification,
                color: ModalColor::Success,
                confirm: {
                    let modal_attributes_state = modal_attributes_state.clone();
                    Callback::from(move |_| { 
                        modal_attributes_state.set(ModalAttributes {open: false, ..Default::default()});
                    })
                },
                ..Default::default()
            });
        })
    }; 
    if playlist.playlist().provider() == "boater" {
        modal_attributes_state.set(ModalAttributes {
            title: String::from("Remove Song from Playlist"),
            content: html! {
                <>{"Are you sure you want to remove "}{song.title}{" by "}{song.artists}{" from this playlist?"}</>
            },
            confirm: remove_song,
            ..Default::default()
        })
    } else {
        modal_attributes_state.set(ModalAttributes {
            title: String::from("Remove Song from Playlist"),
            content: html! {
                <div class="flex flex-col">
                    <p>{"This playlist is linked to "}<strong>{playlist.playlist().provider().clone()}</strong>
                    <br />
                    {"Do you want to convert it to a custom Boater playlist?"}
                    <br />
                    {"*Cancelling will not remove the song"}</p>
                </div>
            },
            modal_type: ModalType::Confirmation,
            color: ModalColor::Warning,
            confirm: remove_song,
            ..Default::default()
        });
    }
}

pub fn get_client_user_obj() -> BoaterUser {
    BoaterUser {
        id: getUserId().unwrap(),
        display_name: getDisplayName(),
        avatar: getUserAvatar()
    }
}

pub fn get_boater_users(ws: WebSocket) {
    ws.send_with_str(
        to_string(&WebSocketPayload {
            action: "getUsers",
            ..Default::default()
        })
        .unwrap()
        .as_str(),
    )
    .unwrap_or_default();
}

pub fn get_boater_user_playlists(ws: WebSocket, id: String) {
    ws.send_with_str(
        to_string(&WebSocketPayload {
            action: "getUserPlaylists",
            data: Some(id),
            ..Default::default()
        })
        .unwrap()
        .as_str(),
    )
    .unwrap()
}

pub fn delete_boater_user_playlist(
    ws: WebSocket,
    modal_attributes_state: UseStateHandle<ModalAttributes>,
    playlist: BoaterUserPlaylist,
    conflict_callback: UseStateHandle<Callback<Value>>,
    close_cb: Callback<ClickEvent>
) {
    modal_attributes_state.set(ModalAttributes {
        title: String::from("Delete Playlist Confirmation"),
        content: html! {
            <div class="flex flex-col gap-[0.5vw]">
                {"Are you sure you want to delete this playlist?"}
                <div class="w-full flex flex-row">
                    <div class="relative h-full aspect-square">
                        <div class="absolute top-[0.2vw] left-[0.2vw] py-[0.2vw] px-[0.4vw] text-[0.8vw] rounded-md bg-neutral-800/70 backdrop-blur z-[1]">
                            {service_icon_html(playlist.provider)}
                        </div>
                        <img class="rounded-l" width="128" height="128" src={format_url(playlist.cover.unwrap_or_default().as_str())} />
                    </div>
                    <div class="w-full p-[0.5vw] text-left text-[1.2vw] flex flex-col bg-neutral-900 rounded-r">
                        <span class="font-semibold">{playlist.title}</span>
                        <span class="text-neutral-400 text-[1vw]">{"By "}{playlist.artist}</span>
                    </div>
                </div>
            </div>
        },
        color: ModalColor::Danger,
        confirm: {
            let ws = ws.clone();
            let modal_attributes_state = modal_attributes_state.clone();
            let id = playlist.id.clone();
            Callback::from(move |_| {
                ws.send_with_str(
                    to_string(&WebSocketPayload {
                        action: "removeUserPlaylist",
                        dataObject: Some(
                            to_value(HashMap::from([
                                ("id", id.as_str()),
                                ("owner", getUserId().unwrap_or_default().as_str()),
                            ]))
                            .unwrap(),
                        ),
                        ..Default::default()
                    })
                    .unwrap()
                    .as_str(),
                )
                .unwrap();
                modal_attributes_state.set(ModalAttributes { open: false, ..Default::default() });
                conflict_callback.set({
                    let ws = ws.clone();
                    let close_cb = close_cb.clone();
                    Callback::from(move |result: Value| {
                        get_boater_users(ws.clone());
                        get_boater_user_playlists(ws.clone(), getUserId().unwrap_or_default());
                        close_cb.emit(ClickEvent::new("").unwrap())
                    })
                });

            })
        },
        ..Default::default()
    })
}

pub fn get_boater_playlist_songs(ws: WebSocket, id: String, owner: String) {
    ws.send_with_str(
        to_string(&WebSocketPayload {
            action: "getPlaylistSongs",
            dataObject: Some(to_value(HashMap::from([("id", id), ("owner", owner)])).unwrap()),
            ..Default::default()
        })
        .unwrap()
        .as_str(),
    )
    .unwrap()
}
