use serde_json::Value;
use web_sys::{MouseEvent, window, HtmlIFrameElement};
use yew::{Callback, function_component, Html, html, use_context, use_state, use_effect};
use yew::html::onload::Event as LoadEvent;
use wasm_bindgen::JsCast;
use crate::lib::util::{format_time, service_icon_html};
use crate::components::modal::{ModalAttributes, ModalColor};
use crate::lib::util::format_url;
use crate::lib::websocket::add_song_to_playlist;
use crate::lib::api::ServiceSong;
use crate::routes::guild::DefaultProps;
use gloo::console::log;

#[function_component(Player)]
pub fn player() -> Html {
    let default_props = use_context::<DefaultProps>().expect("ctx");
    let ws = (*default_props.ws).clone();
    let queue_data = (*default_props.queue_data).clone();
    let modal_attributes_state = default_props.modal_attributes.clone();
    let cover_full = use_state(|| true);
    let toggle_cover_size = {
        let cover_full = cover_full.clone();
        if *cover_full {
            Callback::from(move |_: MouseEvent| cover_full.set(false))
        } else {
            Callback::from(move |_: MouseEvent| cover_full.set(true))
        }
    };

    let default = Value::String("No Data".to_string());
    let song = queue_data.get("song").unwrap_or(&default);
    let interaction = queue_data.get("interaction").unwrap_or(&default);
    let skip = {
        let ws = ws.clone();
        let modal_attributes_state = modal_attributes_state.clone();
        Box::new(move |song: Value| {
            let ws = ws.clone();
            let modal_attributes_state = modal_attributes_state.clone();
            Callback::from(move |_| {
                modal_attributes_state.set(ModalAttributes {
                    title: String::from("Skip Confirmation"),
                    content: html! {
                        <p>
                            {"Are you sure you want to skip the current song"}
                            <br /><strong>
                                {song.get("title").unwrap()}{" by "}{song.get("artist").unwrap().as_str().unwrap()}
                            </strong><br />
                        </p>
                    },
                    color: ModalColor::Danger,
                    confirm: {
                        let ws = ws.clone();
                        let modal_attributes_state = modal_attributes_state.clone();
                        Callback::from(move |_| {
                            ws.send_with_str(r#"{"action":"skip"}"#).unwrap();
                            modal_attributes_state.set(ModalAttributes { open: false, ..Default::default() })
                        })
                    },
                    ..Default::default()
                })
            })
        })
    };
    let skip_in_playlist = {
        let ws = ws.clone();
        let modal_attributes_state = modal_attributes_state.clone();
        Box::new(move |song: Value| {
            let ws = ws.clone();
            let modal_attributes_state = modal_attributes_state.clone();
            Callback::from(move |_| {
                modal_attributes_state.set(ModalAttributes {
                    title: String::from("Skip Confirmation"),
                    content: html! {
                        <p>
                            {"Are you sure you want to skip the current song"}
                            <br /><strong>
                                {song.get("title").unwrap()}{" by "}{song.get("artist").unwrap().as_str().unwrap()}
                            </strong><br />
                            {"in this playlist"}
                        </p>
                    },
                    color: ModalColor::Danger,
                    confirm: {
                        let ws = ws.clone();
                        let modal_attributes_state = modal_attributes_state.clone();
                        Callback::from(move |_| {
                            ws.send_with_str(r#"{"action":"skipInPlaylist"}"#).unwrap();
                            modal_attributes_state.set(ModalAttributes { open: false, ..Default::default() })
                        })
                    },
                    ..Default::default()
                })
            })
        })
    };
    let force_play = {
        let ws = ws.clone();
        let modal_attributes_state = modal_attributes_state.clone();
        Box::new(move |index, song: Value| {
            let ws = ws.clone();
            let modal_attributes_state = modal_attributes_state.clone();
            Callback::from(move |_| {
                modal_attributes_state.set(ModalAttributes {
                    title: String::from("Force Play Confirmation"),
                    content: html! {
                        <p>
                            {"Are you sure you want to force play"}
                            <br /><strong>
                                {song.get("title").unwrap()}{" by "}{song.get("artist").unwrap().as_str().unwrap()}
                            </strong><br />
                            {"from the server's queue?"}
                            <br />
                            <em class="text-[1.2vw]">{"*This will cancel the currently playing song"}</em>
                        </p>
                    },
                    color: ModalColor::Primary,
                    confirm: {
                        let ws = ws.clone();
                        let modal_attributes_state = modal_attributes_state.clone();
                        Callback::from(move |_| {
                            ws.send_with_str(format!(r#"{{
                                "action": "forcePlay",
                                "data": "{index}"
                            }}"#).as_str()).unwrap();
                            modal_attributes_state.set(ModalAttributes { open: false, ..Default::default() });
                        })
                    },
                    ..Default::default()
                })
            })
        })
    };
    let force_play_in_playlist = {
        let ws = ws.clone();
        let modal_attributes_state = modal_attributes_state.clone();
        Box::new(move |index, song: Value| {
            let ws = ws.clone();
            let modal_attributes_state = modal_attributes_state.clone();
            Callback::from(move |_| {
                modal_attributes_state.set(ModalAttributes {
                    title: String::from("Force Play Confirmation (In Playlist)"),
                    content: html! {
                        <p>
                            {"Are you sure you want to force play"}
                            <br /><strong>
                                {song.get("title").unwrap()}{" by "}{song.get("artist").unwrap().as_str().unwrap()}
                            <br /></strong>
                            {"from the current playlist?"}
                            <br />
                            <em class="text-[1.2vw]">{"*This will cancel the currently playing song"}</em>
                        </p>
                    },
                    color: ModalColor::Primary,
                    confirm: {
                        let ws = ws.clone();
                        let modal_attributes_state = modal_attributes_state.clone();
                        Callback::from(move |_| {
                            ws.send_with_str(format!(r#"{{
                                "action": "forcePlayInPlaylist",
                                "data": "{index}"
                            }}"#).as_str()).unwrap();
                            modal_attributes_state.set(ModalAttributes {open: false, ..Default::default()});
                        })
                    },
                    ..Default::default()
                })
            })
        })
    };
    let add_song_to_playlist = {
        let default_props = default_props.clone();
        Box::new(move |mut song: Value, is_playlist: bool| {
            let default_props = default_props.clone();
            if !is_playlist {
                let duration = song.get("durationMs").cloned().unwrap();
                let release = song.get("releaseDate").cloned().unwrap();
                if let Value::Object(ref mut map) = song {
                    map.insert(String::from("duration_ms"), duration);
                    map.insert(String::from("release_date"), release);
                }
            }
            Callback::from(move |_| {
                add_song_to_playlist(default_props.clone(), serde_json::from_value::<ServiceSong>(song.clone()).unwrap())
            })
        })
    };
    let remove = {
        let ws = ws.clone();
        let modal_attributes_state = modal_attributes_state.clone();
        Box::new(move |index, song: Value| {
            let ws = ws.clone();
            let modal_attributes_state = modal_attributes_state.clone();
            Callback::from(move |_| {
                modal_attributes_state.set(ModalAttributes {
                    title: String::from("Remove Confirmation"),
                    content: html! {
                        <p>
                            {"Are you sure you want to remove"}
                            <br /><strong>
                                {song.get("title").unwrap()}{" by "}{song.get("artist").unwrap().as_str().unwrap()}
                            </strong><br />
                            {"from the server's queue?"}
                        </p>
                    },
                    color: ModalColor::Danger,
                    confirm: {
                        let ws = ws.clone();
                        let modal_attributes_state = modal_attributes_state.clone();
                        Callback::from(move |_| {
                            ws.send_with_str(format!(r#"{{
                                "action": "remove",
                                "data": "{}"
                            }}"#, index).as_str()).unwrap();
                            modal_attributes_state.set(ModalAttributes { open: false, ..Default::default() });
                        })
                    },
                    ..Default::default()
                })
            })
        })
    };
    html! {
        <>
            <div class="absolute bottom-0 w-full"> // Song, Artist, Duration Container
                <div class="px-[1vw]"> // Artist Logo
                    <div class="flex items-center w-fit p-[0.6vw] gap-[0.6vw] text-[1.5vw] bg-neutral-900/70 backdrop-blur-lg rounded-lg">
                        <img
                            class="rounded-full h-[2.8vw] aspect-square"
                            src={format!("{}", format_url(song.get("artistLogo").unwrap_or(&default).as_str().unwrap_or(&default.as_str().unwrap())))}
                        />
                        {song.get("artists").unwrap_or(&default).as_str().unwrap()}
                    </div>
                </div>
                <div class="p-[1vw]"> // Song Title
                    <div class="flex flex-col justify-center items-center w-fit p-[0.6vw] text-[2.5vw] bg-neutral-900/70 backdrop-blur-lg rounded-lg">
                        {song.get("title").unwrap_or(&default).as_str().unwrap()}
                        if !queue_data.get("playlist").unwrap_or(&Value::from(None::<Value>)).is_null() {
                            <div class="flex items-center text-[1.2vw] h-[2.5vw] pt-[0.6vw]">
                                {"From "}
                                <img
                                    class="rounded-l-md h-full ml-[0.6vw] aspect-square"
                                    src={format_url(queue_data.get("playlist").unwrap().get("cover").unwrap_or(&Value::from("")).as_str().unwrap())}
                                />
                                <div class="flex items-center w-fit h-full p-[0.6vw] text-[1vw] bg-neutral-950/70 rounded-r-md">
                                    <p><strong>{queue_data.get("playlist").unwrap().get("title").unwrap().as_str().unwrap().to_string()}</strong>
                                    {" By "}{queue_data.get("playlist").unwrap().get("artist").unwrap().as_str().unwrap().to_string()}</p>
                                </div>
                                <div class="relative flex items-center justify-center group w-fit h-full p-[0.6vw] text-[1vw] bg-neutral-950/70 rounded-md ml-[0.6vw]">
                                    {"Song "}{queue_data.get("playlistPosition").unwrap().as_u64().unwrap() + 1}
                                    {" of "}{queue_data.get("playlist").unwrap().get("song_count").unwrap().as_u64().unwrap()}
                                    <i class="fa-solid fa-ellipsis ml-[0.4vw]" />
                                    <div class="tooltip bg-transparent backdrop-blur-none bottom-[100%]">
                                        <div
                                            class="flex flex-col divide-y divide-neutral-700 bg-neutral-800 text-[1vw] max-h-[25.2vw] backdrop-blur-lg rounded-md shadow-md 
                                            overflow-hidden overflow-y-auto leading-[1.2vw]"
                                        >
                                            { for queue_data.get("playlistSongs").unwrap().as_array().unwrap().iter().enumerate().map(|(index, pl_song)| {
                                                let current_play = index as u64 == queue_data.get("playlistPosition").unwrap().as_u64().unwrap();
                                                html! {
                                                    <div
                                                        class={format!("flex flex-row rounded-md h-[5vw] p-[0.6vw] {}",
                                                            if current_play {"sticky top-0 bottom-0 bg-green-800 z-20"} else {"bg-neutral-800"})}
                                                    >
                                                        <div class="relative h-full aspect-square">
                                                            <img class="h-full aspect-square rounded" src={format_url(pl_song.get("cover").unwrap().as_str().unwrap())} />
                                                            <div
                                                                class="flex items-center justify-center absolute top-0 left-0 h-full w-full text-[1.6vw] 
                                                                bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-neutral-800/60 from-0% to-neutral-800/0 to-100% size-24"
                                                            >
                                                                <strong>{index + 1}</strong>
                                                            </div>
                                                        </div>
                                                        <div class="w-full flex flex-col items-center p-[0.6vw]">
                                                            if current_play {
                                                                <span class="text-[0.8vw] -mt-[0.6vw] font-light">{"Currently Playing"}</span>
                                                            }
                                                            <span>{pl_song.get("title").unwrap().as_str().unwrap().to_string()}</span>
                                                            <span class="text-neutral-400 text-[0.8vw]">{"By "}{pl_song.get("artist").unwrap().as_str().unwrap().to_string()}</span>
                                                        </div>
                                                        <div class="h-full aspect-square flex justify-center items-center">
                                                        if !current_play {
                                                            <button class="h-1/2 aspect-square rounded-md bg-neutral-700" onclick={force_play_in_playlist(index, pl_song.clone())}>
                                                                <i class="fa-solid fa-play" />
                                                            </button>
                                                        } else {
                                                            <button class="h-1/2 aspect-square rounded-md bg-neutral-700" onclick={skip_in_playlist(pl_song.clone())}>
                                                                <i class="fa-solid fa-forward-step" />
                                                            </button>
                                                        }
                                                        </div>
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    </div>
                </div>
                <div class="overflow-hidden"> // Duration Bar
                    <div class="absolute h-[1vw] w-full bg-neutral-800"></div>
                    <div
                        class="h-[1vw] bg-sky-400 transition duration-150 ease-out"
                        style={format!("transform: translateX({}%)", -100.0 + (
                            (queue_data.get("position").unwrap_or(&Value::from(0)).as_f64().unwrap() / song.get("durationMs").unwrap_or(&Value::from(100)).as_f64().unwrap()) * 100.0
                        ))}
                    ></div>
                </div>
            </div>
            <div class="absolute flex items-center justify-center h-full w-full -z-10 overflow-hidden"> // Song Cover
                <img
                    class={format!("object-cover opacity-75 {}", if *cover_full {"w-full"} else {"h-full"})}
                    src={format!("{}", format_url(song.get("cover").unwrap_or(&default).as_str().unwrap()))}
                />
            </div>
            <div class="absolute right-0 h-full p-[1vw] pt-[5.5vw] pb-[2vw]"> // Song Queue
                <div class="relative h-full w-[21vw] flex flex-col gap-[1vw]">
                    if queue_data.get("queue").is_some() {
                        { for queue_data.get("queue").unwrap().as_array().unwrap().iter().enumerate().map(|(index, data)| {
                            let is_playlist = !data.get("playlist").unwrap().is_null();
                            let song = if is_playlist {data.get("playlist").unwrap().clone()} else {data.get("song").unwrap().clone()};
                                html! {
                                    <div class="relative flex w-full h-[6vw] bg-neutral-900/70 backdrop-blur rounded-lg shadow">
                                        <div class="relative h-full aspect-square"> // Song Cover Overlays
                                            <div class="absolute top-[0.2vw] left-[0.2vw] py-[0.2vw] px-[0.4vw] text-[1.2vw] font-semibold rounded-md bg-neutral-800/70 backdrop-blur z-10">
                                                {index + 1}
                                            </div>
                                            <div class="absolute bottom-[0.2vw] right-[0.2vw] py-[0.2vw] px-[0.4vw] text-[1vw] rounded-md bg-neutral-800/70 backdrop-blur z-10">
                                            if !is_playlist {
                                                {format_time(song.get("durationMs").unwrap().as_u64().unwrap())}
                                            } else {
                                                <div class="relative group flex justify-center items-center">
                                                    <i class="fa-solid fa-layer-group mr-[0.4vw]" />
                                                    {song.get("song_count").unwrap().as_u64().unwrap()}
                                                    <span class="tooltip text-[0.8vw] bottom-[120%]">{song.get("song_count").unwrap().as_u64().unwrap()}{" songs"}</span>
                                                </div>
                                            }
                                            </div>
                                            <img class="rounded-lg object-cover h-full opacity-75 shadow-sm" src={format_url(song.get("cover").unwrap_or(&Value::from("")).as_str().unwrap())} />
                                        </div>
                                        <div class="flex w-full flex-col p-[0.4vw] justify-between overflow-hidden">
                                            <div class="flex w-full h-full justify-between">
                                                <p class="text-[1vw] font-semibold line-clamp-2 text-ellipsis overflow-hidden shrink">{song.get("title").unwrap().as_str().unwrap()}</p>
                                                <div class="flex flex-row h-[1.8vw] items-center shrink-0">
                                                    <img
                                                        class="h-full aspect-square rounded-full mr-[0.2vw]"
                                                        src={format_url(data.get("interaction").unwrap().get("avatarUrl").unwrap().as_str().unwrap())}
                                                    />
                                                    <span class="text-[1.2vw]">{service_icon_html(song.get("provider").unwrap().as_str().unwrap().to_string())}</span>
                                                </div>
                                            </div>
                                            <div class="flex gap-[0.6vw] w-full h-fit justify-between">
                                                <p class="self-end text-[0.9vw] w-full text-neutral-400 font-semibold text-nowrap text-ellipsis overflow-hidden">
                                                    {song.get("artist").unwrap().as_str().unwrap()}
                                                </p>
                                                <div class="flex text-[1vw] justify-self-end">
                                                </div>
                                            </div>
                                        </div>
                                        <div class="flex flex-col text-[0.8vw]">
                                            <button
                                                class="h-full bg-neutral-800/70 hover:bg-sky-950/70 hover:text-sky-500 py-[0.4vw] px-[0.6vw] rounded-tr-md transition-colors duration-150"
                                                onclick={force_play(index, song.clone())}
                                            >
                                                <i class="fa-solid fa-play" />
                                            </button>
                                            if !is_playlist {
                                                <button
                                                    class="h-full bg-neutral-800/70 hover:bg-green-950/70 hover:text-green-500 py-[0.4vw] px-[0.6vw] transition-colors duration-150"
                                                    onclick={add_song_to_playlist(song.clone(), is_playlist)}
                                                >
                                                    <i class="fa-solid fa-plus" />
                                                </button>
                                            } else {
                                                <button
                                                    class="h-full bg-neutral-800/70 hover:bg-green-950/70 hover:text-green-500 py-[0.4vw] px-[0.6vw] transition-colors duration-150"
                                                    onclick={add_song_to_playlist(song.clone(), is_playlist)}
                                                >
                                                    <i class="fa-solid fa-save" />
                                                </button>
                                            }
                                            <button
                                                class="h-full bg-neutral-800/70 hover:bg-red-950/70 hover:text-red-500 py-[0.4vw] px-[0.6vw] rounded-br-md transition-colors duration-150"
                                                onclick={remove(index, song.clone())}
                                            >
                                                <i class="fa-solid fa-trash" />
                                            </button>
                                        </div>
                                    </div>
                                }
                            })
                        }
                    }
                    // Queue Controls
                    <div class="absolute flex flex-row justify-between gap-[1vw] bottom-0 h-[5vw] p-[1vw] w-full bg-neutral-900/70 backdrop-blur-lg rounded-lg text-[3vw]">
                        <button class="relative h-fit flex items-center justify-center group">
                            <img class="inline w-[3vw] aspect-square rounded-full" src={format_url(interaction.get("avatarUrl").unwrap_or(&default).as_str().unwrap())} />
                            <span class="tooltip bottom-[110%]">{format!("Requested by {}", interaction.get("name").unwrap_or(&default).as_str().unwrap())}</span>
                        </button>
                        <button
                            class="relative h-fit flex items-center justify-center group hover:text-sky-400"
                            onclick={let ws = ws.clone(); Callback::from(move |_| ws.send_with_str(r#"{"action": "togglePlay"}"#).unwrap())}
                        >
                            if queue_data.get("isPaused").unwrap_or(&Value::Bool(false)).as_bool().unwrap() {
                                <i class="fa-solid fa-play" />
                                <span class="tooltip bottom-[110%]">{"Resume"}</span>
                            } else {
                                <i class="fa-solid fa-pause" />
                                <span class="tooltip bottom-[110%]">{"Pause"}</span>
                            }
                        </button>
                        <button class="relative h-fit flex items-center justify-center group hover:text-red-500" onclick={skip(song.clone())}>
                            <i class="fa-solid fa-forward-step" />
                            <span class="tooltip bottom-[110%]">{"Skip"}</span>
                        </button>
                        <button class="relative h-fit flex items-center justify-center group hover:text-green-400">
                            <i class="fa-solid fa-repeat" />
                            <span class="tooltip bottom-[110%]">{"Loop"}</span>
                        </button>
                        <button class="relative h-fit flex items-center justify-center group hover:text-sky-400">
                            <i class="fa-solid fa-ellipsis" />
                            <span class="tooltip bottom-[110%]">{"More"}</span>
                        </button>
                    </div>
                </div>
            </div>
        </>
    }
}
