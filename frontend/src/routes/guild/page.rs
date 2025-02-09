use crate::components::player::Player;
use crate::lib::api::ServiceSong;
use crate::lib::util::{format_release, format_time, format_url};
use crate::lib::websocket::{add_queue_song, add_song_to_playlist};
use crate::routes::guild::DefaultProps;
use gloo::net::http::Request;
use gloo::timers::callback::Timeout;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use web_sys::{window, InputEvent};
use yew::html::onchange::Event;
use yew::platform::spawn_local;
use yew::{
    function_component, html, use_context, use_mut_ref, use_state, Callback, Html, TargetCast,
};

#[function_component(GuildHome)]
pub fn guild_home() -> Html {
    let default_props = use_context::<DefaultProps>().expect("ctx");
    let ws = (*default_props.ws).clone();
    let queue_data = (*default_props.queue_data).clone();
    let modal_attributes_state = default_props.modal_attributes.clone();

    let provider = use_state(|| String::from("spotify"));
    let songs_list = use_state(Vec::<ServiceSong>::new);
    let debounce_handle = use_mut_ref(|| None);
    let search_songs = {
        let provider = (*provider).clone();
        let songs_list = songs_list.clone();
        Callback::from(move |e: InputEvent| {
            let provider = provider.clone();
            let songs_list = songs_list.clone();
            let input = e.target_unchecked_into::<HtmlInputElement>();
            if input.value() == "" {
                songs_list.set(Vec::<ServiceSong>::new());
                return;
            };
            let request = Timeout::new(500, || {
                spawn_local(async move {
                    if input.value() == "" {
                        songs_list.set(Vec::<ServiceSong>::new());
                        return;
                    };
                    let hostname = window()
                        .expect_throw("no window")
                        .location()
                        .hostname()
                        .expect_throw("no hostname");
                    songs_list.set(Request::get(format!("https://{hostname}/.proxy/api/search?provider={provider}&data_type=songs&search={}", input.value()).as_str())
                    .send().await.unwrap().json::<Vec<ServiceSong>>().await.unwrap()
                )
                })
            });
            *debounce_handle.borrow_mut() = Some(request)
        })
    };
    let add_song_to_playlist = {
        let default_props = default_props.clone();
        Box::new(move |song: ServiceSong| {
            let default_props = default_props.clone();
            Callback::from(move |_| {
                add_song_to_playlist(
                    default_props.clone(),
                    song.clone()
                )
            })
        })
    };
    html! {
        <div class="w-full h-full flex flex-col">
            <div class="absolute left-[20vw] right-0 p-[1vw] z-10"> // Search Bar
                <div class="relative">
                    <form class={format!("bg-neutral-900/70 backdrop-blur-lg shadow-lg flex text-[1.5vw] {}", if !(*songs_list).is_empty() {"rounded-t-xl rounded-br-xl"} else {"rounded-xl"})}>
                        <input id="song-search" type="search" class="w-full appearance-none bg-transparent p-[0.6vw] outline-none" oninput={search_songs} placeholder="Search Song" />
                        <div class="w-[30vw] p-[0.6vw] bg-neutral-800/70 backdrop-blur-lg rounded-r-xl flex grow-0 z-50">
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
                    if !(*songs_list).is_empty() {
                        <div class="absolute flex left-0 w-[56.2vw]">
                            <div class="max-h-[30vw] h-fit w-full p-[0.6vw] flex flex-col gap-[0.6vw] bg-neutral-900/70 backdrop-blur-lg rounded-b-xl overflow-y-scroll snap-y z-30">
                                { for songs_list.iter().map(|song| { // Song Search Results
                                        html! {
                                            <div class="flex bg-neutral-900 backdrop-blur shadow-md shadow-inner rounded-lg snap-center">
                                                <div class="relative h-[9.3vw] aspect-square">
                                                    <div class="absolute bg-neutral-800/70 backdrop-blur p-[0.2vw] rounded-md bottom-[0.2vw] right-[0.2vw] text-[1.2vw]">
                                                        {format_time(song.duration_ms)}
                                                    </div>
                                                    <img class="h-full aspect-square rounded-l-lg" src={format_url(&song.cover)} />
                                                </div>
                                                <div class="flex gap-[1vw] justify-between w-full p-[0.6vw]">
                                                    <div>
                                                        <p class="text-[1.5vw] font-semibold">{&song.title}</p>
                                                        <p class="text-neutral-400 text-[1vw] font-semibold">{format!("Released: {}", format_release(song.release_date.as_str()))}</p>
                                                    </div>
                                                    <div class="flex flex-col justify-between items-end h-full text-[1.2vw] text-right font-semibold">
                                                        {&song.artists}
                                                        <div class="flex flex-row">
                                                            <button class="w-fit p-[0.6vw] bg-neutral-800 text-[1.5vw] rounded-l-md hover:bg-sky-950 hover:text-sky-500 transition-colors duration-150" onclick={let ws = ws.clone(); let modal_attributes_state = modal_attributes_state.clone(); let song = song.clone(); Callback::from(move |_| {add_queue_song(ws.clone(), modal_attributes_state.clone(), song.clone())})}>
                                                                <i class="fa-solid fa-play px-[0.4vw]" />
                                                            </button>
                                                            <button class="w-fit p-[0.6vw] bg-neutral-800 text-[1.5vw] rounded-r-md hover:bg-green-950 hover:text-green-500 transition-colors duration-150" onclick={add_song_to_playlist(song.clone())}>
                                                                <i class="fa-solid fa-plus px-[0.4vw]" />
                                                            </button>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    })
                                }
                            </div>
                        </div>
                    }
                </div>
            </div>
            <div class="relative h-full w-full"> // Player
            if ws.ready_state() != 1 {
                <div class="flex flex-col w-full h-full text-center justify-center font-semibold animate-pulse bg-neutral-950">
                    <h3 class="text-[4vw]">{"Connecting..."}</h3>
                </div>
            } else if queue_data.is_empty() {
                <div class="flex flex-col h-full w-full text-center justify-center font-semibold">
                    <h2 class="text-[4vw] font-semibold">{"Queue does not exist"}</h2><br /><h3 class="text-[2.5vw]">{"Use /play or the search bar to start a queue"}</h3>
                </div>
            } else {
                <Player />
            }
            </div>
        </div>
    }
}
