use crate::components::generic_playlist::{GenericPlaylist, GenericPlaylistComponent, GenericSong};
use crate::lib::api::{playlist_songs_request, ServicePlaylist, ServiceSong};
use crate::lib::util::format_url;
use gloo::net::http::Request;
use gloo::timers::callback::Timeout;
use serde_json::Value;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{window, HtmlElement, HtmlInputElement, InputEvent};
use yew::html::onchange::Event;
use yew::html::onscroll::Event as ScrollEvent;
use yew::platform::spawn_local;
use yew::{
    function_component, html, use_effect_with, use_mut_ref, use_state, Callback, Html, TargetCast,
};

#[function_component(PlaylistsSearch)]
pub fn search() -> Html {
    let provider = use_state(|| String::from("spotify"));
    let playlists_data = use_state(Vec::<ServicePlaylist>::new);
    let selected_playlist = use_state(|| None::<ServicePlaylist>);
    let songs = use_state(|| None::<Vec<ServiceSong>>);
    let song_count = use_state(|| 0);
    let debounce_handle = use_mut_ref(|| None);
    let search_playlists = {
        let provider = (*provider).clone();
        let playlists_data = playlists_data.clone();
        Callback::from(move |e: InputEvent| {
            let provider = provider.clone();
            let playlists_data = playlists_data.clone();
            let input = e.target_unchecked_into::<HtmlInputElement>();
            if input.value() == "" {
                playlists_data.set(Vec::<ServicePlaylist>::new());
                return;
            };
            let request = Timeout::new(500, || {
                spawn_local(async move {
                    if input.value() == "" {
                        playlists_data.set(Vec::<ServicePlaylist>::new());
                        return;
                    };
                    let hostname = window()
                        .expect_throw("no window")
                        .location()
                        .hostname()
                        .expect_throw("no hostname");
                    playlists_data.set(Request::get(format!("https://{hostname}/.proxy/api/search?provider={provider}&data_type=playlists&search={}", input.value()).as_str())
                        .send().await.unwrap().json::<Vec<ServicePlaylist>>().await.unwrap()
                    )
                })
            });
            *debounce_handle.borrow_mut() = Some(request)
        })
    };
    let load_more_songs_debounce = use_mut_ref(|| None);
    let load_more_songs = {
        let provider = (*provider).clone();
        let songs = songs.clone();
        let song_count = song_count.clone();
        let selected_playlist = (*selected_playlist).clone();
        Callback::from(move |event: ScrollEvent| {
            if (*songs).clone().unwrap().len() as u64 != *song_count {
                let element = event.target().unwrap().dyn_into::<HtmlElement>().unwrap();
                if element.offset_height() + element.scroll_top() + 1 >= element.scroll_height() {
                    let provider = provider.clone();
                    let songs_state = songs.clone();
                    let songs = (*songs_state).clone().unwrap();
                    let song_count = song_count.clone();
                    let selected_playlist = selected_playlist.clone().unwrap();
                    *load_more_songs_debounce.borrow_mut() = Some(Timeout::new(500, move || {
                        playlist_songs_request(
                            provider,
                            Callback::from(move |new_songs| {
                                element.scroll_to_with_x_and_y(
                                    0.0,
                                    (element.scroll_height() - element.offset_height() - 20) as f64,
                                );
                                songs_state.set(Some(new_songs))
                            }),
                            song_count.clone(),
                            selected_playlist,
                            songs,
                            false,
                        )
                    }))
                }
            }
        })
    };
    let load_all_songs = {
        let provider = (*provider).clone();
        let songs = songs.clone();
        let song_count = song_count.clone();
        Callback::from(
            move |(playlist, data_callback): (ServicePlaylist, Callback<Vec<ServiceSong>>)| {
                let songs = songs.clone();
                let set_songs = {
                    let songs_state = songs.clone();
                    Callback::from(move |songs: Vec<ServiceSong>| {
                        songs_state.set(Some(songs.clone()));
                        data_callback.emit(songs)
                    })
                };
                playlist_songs_request(
                    provider.clone(),
                    set_songs,
                    song_count.clone(),
                    playlist,
                    (*songs).clone().unwrap(),
                    true,
                )
            },
        )
    };
    let close_playlist = {
        let selected_playlist = selected_playlist.clone();
        let songs = songs.clone();
        Callback::from(move |_| {
            selected_playlist.set(None);
            songs.set(None)
        })
    };
    use_effect_with((*selected_playlist).clone(), {
        let provider = (*provider).clone();
        let songs = songs.clone();
        let song_count = song_count.clone();
        move |playlist| {
            if playlist.is_none() {
                return;
            }
            let playlist = playlist.clone();
            let songs = songs.clone();
            let song_count = song_count.clone();
            spawn_local(async move {
                let mut playlist = playlist.unwrap().clone();
                if playlist.song_count.is_none() {
                    playlist.song_count = Some((*songs).clone().unwrap_or(Vec::new()).len() as u64)
                }
                let hostname = window()
                    .expect_throw("no window")
                    .location()
                    .hostname()
                    .expect_throw("no hostname");
                let response = Request::get(format!("https://{hostname}/.proxy/api/search?provider={provider}&data_type=playlist_songs&search={}&offset={}", playlist.id, (*songs).clone().unwrap_or_default().len()).as_str())
                    .send().await.unwrap().json::<Value>().await.unwrap();
                let received_songs = response.get("songs").unwrap().clone();
                songs.set(Some(
                    serde_json::from_value(received_songs.clone()).unwrap(),
                ));
                if response.get("next").unwrap().is_null() {
                    song_count.set(received_songs.as_array().unwrap().len() as u64)
                } else {
                    song_count.set((received_songs.as_array().unwrap().len() + 1) as u64)
                }
            })
        }
    });
    html! {
        <div class="h-full flex flex-col gap-[1vw]">
            if selected_playlist.is_none() {
                <form class="backdrop-blur-lg shadow-lg flex text-[1.5vw]">
                    <input id="song-search" type="search" class="w-full appearance-none bg-neutral-800 rounded-l-xl p-[0.6vw] outline-none" oninput={search_playlists} placeholder="Search Playlists (Use id at the end of URL to find a specific playlist)" />
                    <div class="w-[30vw] p-[0.6vw] bg-neutral-800/70 backdrop-blur-lg rounded-r-xl flex grow-0">
                        <label class="w-fit text-nowrap" for="provider">{"Audio Service:"}</label>
                        <select class="text-center w-full bg-transparent appearance-none" name="provider" onchange={let provider = provider.clone(); Callback::from(move |e: Event| {provider.set(e.target().expect("no value").unchecked_into::<HtmlInputElement>().value())})}>
                            <option class="bg-neutral-900" value="spotify" selected=true >{"Spotify"}</option>
                            <option class="bg-neutral-900" value="apple">{"Apple Music"}</option>
                            <option class="bg-neutral-900" value="youtube">{"YouTube"}</option>
                            <option class="bg-neutral-900" value="soundcloud">{"SoundCloud"}</option>
                            <option class="bg-neutral-900" value="deezer">{"Deezer"}</option>
                        </select>
                    </div>
                </form>
                <div class="h-full grid grid-cols-5 grid-rows-2 grid-flow-row auto-rows-max overflow-auto">
                    { for playlists_data.clone().iter().map(|playlist| {
                        html! {
                            <div class="col-span-1 h-full py-[0.5vh] px-[1.2vw] text-[1vw]">
                                <button class="h-full w-full group" onclick={let playlist = playlist.clone(); let selected_playlist = selected_playlist.clone(); Callback::from(move |_| selected_playlist.set(Some(playlist.clone())))}>
                                    <img class="rounded-t-md w-full aspect-square" src={format_url(playlist.cover.clone().unwrap_or_default().as_str())} />
                                    <div class="p-[0.4vw] h-fit w-full bg-neutral-800 rounded-b-md flex flex-col text-left">
                                        <span class="text-nowrap text-ellipsis overflow-hidden group-hover:text-sky-500 duration-150">{&playlist.title}</span>
                                        <span class="text-neutral-400 text-nowrap text-ellipsis overflow-hidden">{"By "}{&playlist.artist}</span>
                                    </div>
                                </button>
                            </div>
                        }
                    })}
                </div>
            } else {
                <GenericPlaylistComponent
                    selected_playlist={GenericPlaylist::from(ServicePlaylist { song_count: Some(*song_count), ..(*selected_playlist).clone().unwrap() })}
                    songs={(*songs).clone().map(|songs| songs.into_iter().map(Into::into).collect::<Vec<GenericSong>>())}
                    update={Some(load_more_songs)}
                    load_all={Some(load_all_songs)}
                    close={close_playlist}
                />
            }
        </div>
    }
}
