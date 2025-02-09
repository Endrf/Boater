use crate::getUserId;
use crate::lib::api::{ServicePlaylist, ServiceSong};
use crate::lib::util::{format_time, format_url};
use crate::lib::websocket::{
    add_queue_song, add_queue_playlist, add_song_to_playlist, remove_song_from_playlist, delete_boater_user_playlist, save_playlist, BoaterSong, BoaterUserPlaylist,
};
use crate::routes::guild::DefaultProps;
use yew::html::onclick::Event as ClickEvent;
use yew::html::onscroll::Event;
use yew::{function_component, html, use_state, use_mut_ref, use_context, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub selected_playlist: GenericPlaylist,
    pub songs: Option<Vec<GenericSong>>,
    pub update: Option<Callback<Event>>,
    pub load_all: Option<Callback<(ServicePlaylist, Callback<Vec<ServiceSong>>)>>,
    pub close: Callback<ClickEvent>,
}

pub trait PlaylistData {
    fn id(&self) -> String;
    fn title(&self) -> String;
    fn artist(&self) -> String;
    fn cover(&self) -> Option<String>;
    fn song_count(&self) -> Option<u64>;
    fn provider(&self) -> String;
}

#[derive(Clone, PartialEq)]
pub enum GenericPlaylist {
    ExternalPlaylist(ServicePlaylist),
    BoaterPlaylist(BoaterUserPlaylist),
}

impl GenericPlaylist {
    pub fn playlist(&self) -> &dyn PlaylistData {
        match self {
            GenericPlaylist::ExternalPlaylist(p) => p,
            GenericPlaylist::BoaterPlaylist(p) => p,
        }
    }
}

impl From<BoaterUserPlaylist> for GenericPlaylist {
    fn from(value: BoaterUserPlaylist) -> Self {
        GenericPlaylist::BoaterPlaylist(value)
    }
}

impl From<ServicePlaylist> for GenericPlaylist {
    fn from(value: ServicePlaylist) -> Self {
        GenericPlaylist::ExternalPlaylist(value)
    }
}

impl From<BoaterUserPlaylist> for ServicePlaylist {
    fn from(value: BoaterUserPlaylist) -> Self {
        ServicePlaylist {
            id: value.id,
            title: value.title,
            artist: value.artist,
            cover: value.cover,
            song_count: Some(value.song_count),
            provider: value.provider
        }
    }
}

impl From<GenericPlaylist> for ServicePlaylist {
    fn from(value: GenericPlaylist) -> Self {
        match value {
            GenericPlaylist::ExternalPlaylist(p) => p,
            GenericPlaylist::BoaterPlaylist(p) => p.into(),
        }
    }
}

pub trait SongData {
    fn title(&self) -> String;
    fn artists(&self) -> String;
    fn id(&self) -> String;
    fn url(&self) -> String;
    fn cover(&self) -> String;
    fn duration_ms(&self) -> u64;
    fn release_date(&self) -> String;
    fn provider(&self) -> String;
}

#[derive(Clone, PartialEq)]
pub enum GenericSong {
    ExternalSong(ServiceSong),
    BoaterSong(BoaterSong),
}

impl GenericSong {
    fn song(&self) -> &dyn SongData {
        match self {
            GenericSong::ExternalSong(p) => p,
            GenericSong::BoaterSong(p) => p,
        }
    }
}

impl From<ServiceSong> for GenericSong {
    fn from(value: ServiceSong) -> Self {
        GenericSong::ExternalSong(value)
    }
}

impl From<GenericSong> for ServiceSong {
    fn from(value: GenericSong) -> Self {
        match value {
            GenericSong::ExternalSong(p) => p,
            GenericSong::BoaterSong(p) => p.into(),
        }
    }
}

impl From<BoaterSong> for GenericSong {
    fn from(value: BoaterSong) -> Self {
        GenericSong::BoaterSong(value)
    }
}

#[function_component(GenericPlaylistComponent)]
pub fn generic_playlist_component(props: &Props) -> Html {
    let default_props = use_context::<DefaultProps>().expect("ctx");
    let selected_playlist = props.selected_playlist.playlist();
    let save_playlist = {
        let default_props = default_props.clone();
        let selected_playlist = props.selected_playlist.clone();
        let load_all = props.load_all.clone();
        Callback::from(move |_| {
            if let GenericPlaylist::ExternalPlaylist(playlist) = selected_playlist.clone() {
                save_playlist(
                    (*default_props.ws).clone(),
                    default_props.modal_attributes.clone(),
                    playlist.clone(),
                    default_props.db_response_callback.clone(),
                    load_all.clone().unwrap(),
                )
            }
        })
    };
    let delete_playlist = {
        let default_props = default_props.clone();
        let selected_playlist = props.selected_playlist.clone();
        let close = props.close.clone();
        Callback::from(move |_| {
            if let GenericPlaylist::BoaterPlaylist(playlist) = selected_playlist.clone() {
                delete_boater_user_playlist(
                    (*default_props.ws).clone(),
                    default_props.modal_attributes.clone(),
                    playlist.clone(),
                    default_props.db_response_callback.clone(),
                    close.clone()
                )
            }
        })
    };
    let play_playlist = {
        let default_props = default_props.clone();
        let selected_playlist = props.selected_playlist.clone();
        let load_all = if let Some(load_all) = props.load_all.clone() {
            load_all
        } else {
            let songs = props.songs.clone();
            Callback::from(move |(playlist, data_callback): (ServicePlaylist, Callback<Vec<ServiceSong>>)| {
                data_callback.emit(songs.clone().map(|songs| songs.into_iter().map(ServiceSong::from).collect::<Vec<ServiceSong>>()).unwrap());
            })
        };
        Callback::from(move |_| {
            add_queue_playlist(
                (*default_props.ws).clone(),
                default_props.modal_attributes.clone(),
                selected_playlist.clone().into(),
                load_all.clone()
            )
        })
    };
    let play_song = {
        let default_props = default_props.clone();
        Box::new(move |generic_song: GenericSong| {
            let default_props = default_props.clone();
            Callback::from(move |_| {
                add_queue_song(
                    (*default_props.ws).clone(),
                    default_props.modal_attributes.clone(),
                    generic_song.clone().into()
                )
            })
        })
    };
    let add_song_to_playlist = {
        let default_props = default_props.clone();
        Box::new(move |generic_song: GenericSong| {
            let default_props = default_props.clone();
            Callback::from(move |_| {
                add_song_to_playlist(
                    default_props.clone(),
                    generic_song.clone().into()
                )
            })
        })
    };
    let remove_song_from_playlist = {
        let default_props = default_props.clone();
        let playlist = props.selected_playlist.clone();
        Box::new(move |generic_song: GenericSong, position: usize| {
            let default_props = default_props.clone();
            let playlist = playlist.clone();
            Callback::from(move |_| {
                remove_song_from_playlist(
                    default_props.clone(),
                    playlist.clone(),
                    generic_song.clone().into(),
                    position
                )
            })
        })
    };
    html! {
        <div class="flex flex-col w-full h-full gap-[1vw]">
            <div class="w-full">
                <button onclick={props.close.clone()}><i class="fa-solid fa-chevron-left" /></button>
            </div>
            <div class="flex flex-row items-start h-full overflow-hidden">
                <div class="w-[20vw] text-center">
                    <div class="bg-neutral-800 rounded-md flex flex-col gap-[0.4vw] p-[0.5vw] text-[1.2vw]">
                        <img class="w-full aspect-square rounded-md" src={format_url(selected_playlist.cover().unwrap_or_default().as_str())} />
                        <span class="p-[0.2vw] font-semibold rounded-md">{selected_playlist.title()}</span>
                        <button class="w-full p-[0.2vw] bg-neutral-900 rounded-md hover:bg-sky-950/75 hover:text-sky-500 duration-150" onclick={play_playlist}>
                            <i class="fa-solid fa-play mr-[0.5vw]" />
                            {"Play"}
                        </button>
                        if let GenericPlaylist::ExternalPlaylist(_) = props.selected_playlist.clone() {
                            <button class="w-full p-[0.2vw] bg-neutral-900 rounded-md hover:bg-green-950/75 hover:text-green-500 duration-150" onclick={save_playlist}>
                                <i class="fa-solid fa-floppy-disk mr-[0.5vw]" />
                                {"Save"}
                            </button>
                        } else if let GenericPlaylist::BoaterPlaylist(playlist) = props.selected_playlist.clone() {
                            if playlist.owner == getUserId().unwrap_or_default() {
                                <button class="w-full p-[0.2vw] bg-neutral-900 rounded-md hover:bg-red-950/75 hover:text-red-500 duration-150" onclick={delete_playlist}>
                                    <i class="fa-solid fa-trash mr-[0.5vw]" />
                                    {"Delete"}
                                </button>
                            }
                        }
                    </div>
                </div>
                <div class="w-full h-full pl-[1vw] flex flex-row flex-wrap content-start overflow-y-auto snap-y snap-mandatory" onscroll={&props.update.clone().unwrap_or_default()}>
                    if props.songs.is_none() {
                        <div class="w-full h-full flex justify-center items-center rounded-md bg-neutral-800 animate-pulse"><span class="font-semibold text-[2vw] text-neutral-500">{"Loading Songs..."}</span></div>
                    } else {
                        {for props.songs.clone().unwrap().iter().enumerate().map(|(index, generic_song)| {
                            let song = generic_song.song();
                            html! {
                                <div class="relative w-1/2 h-[18.75%] mb-[1vh] px-[0.25vw] flex flex-row snap-end text-[1vw]">
                                    <div class="relative h-full aspect-square">
                                        <span class="absolute top-[0.2vw] left-[0.2vw] py-[0.2vw] px-[0.4vw] text-[1.2vw] font-semibold rounded-md bg-neutral-900/70">{index + 1}</span>
                                        <span class="absolute bottom-[0.2vw] right-[0.2vw] py-[0.2vw] px-[0.4vw] text-[1vw] font-semibold rounded-md bg-neutral-900/70">{format_time(song.duration_ms())}</span>
                                        <img class="aspect-square rounded-l" src={format_url(song.cover().as_str())} />
                                    </div>
                                    <div class="flex flex-col p-[0.4vw] justify-between overflow-hidden bg-neutral-800 rounded-r w-full">
                                        <span class="font-semibold line-clamp-2 text-ellipsis overflow-hidden">{song.title()}</span>
                                        <div class="flex flex-row justify-between w-full text-[1vw] items-start">
                                            <span class="text-neutral-400 text-nowrap text-ellipsis overflow-hidden">{song.artists()}</span>
                                            <div class="flex flex-row">
                                                <button class="px-[0.4vw] py-[0.2vw] flex items-center justify-center text-left bg-neutral-900 hover:bg-green-950 hover:text-green-500 rounded aspect-square" onclick={play_song(generic_song.clone())}>
                                                    <i class="fa-solid fa-play" />
                                                </button>
                                                <button class="px-[0.4vw] py-[0.2vw] relative bg-neutral-900 hover:bg-sky-950 rounded flex items-center justify-center aspect-square hover:text-sky-500 group">
                                                    <i class="fa-solid fa-ellipsis" />
                                                    <span class="tooltip flex flex-col rounded bottom-[100%] right-0 p-[0.2vw] text-[0.8vw]">
                                                        <button class="px-[0.4vw] py-[0.2vw] text-left hover:bg-neutral-800 hover:text-green-500 rounded" onclick={add_song_to_playlist(generic_song.clone())}>
                                                            <i class="fa-solid fa-plus mr-[0.2vw]" />
                                                            {"Add to Playlist"}
                                                        </button>
                                                        if let GenericPlaylist::BoaterPlaylist(_) = props.selected_playlist.clone() {
                                                            <button class="px-[0.4vw] py-[0.2vw] text-left hover:bg-neutral-800 hover:text-red-500 rounded" onclick={remove_song_from_playlist(generic_song.clone(), index)}>
                                                                <i class="fa-solid fa-trash mr-[0.2vw]" />
                                                                {"Delete from Playlist"}
                                                            </button>
                                                        }
                                                    </span>
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                        })}
                    }
                    if let GenericPlaylist::ExternalPlaylist(_) = props.selected_playlist.clone() {
                        if props.songs.is_some() && selected_playlist.song_count().is_some() && props.songs.clone().expect("songs").len() as u64 != selected_playlist.song_count().unwrap() && !props.songs.clone().expect("songs").is_empty() {
                            <div class="w-full flex items-center justify-center h-[18.75%] bg-neutral-950 text-[1vw] font-semibold p-[0.5vw] snap-end snap-always rounded">{"Scroll Down to Load More Songs in Playlist"}</div>
                            <div class="w-full h-[0.5vw] snap-start"></div>
                        }
                    }
                </div>
            </div>
        </div>
    }
}
