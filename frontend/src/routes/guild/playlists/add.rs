use crate::components::generic_playlist::{GenericPlaylist, GenericPlaylistComponent, GenericSong};
use crate::components::modal::{ModalAttributes, ModalColor, ModalType};
use crate::lib::api::{
    api_link_request, user_service_playlist_songs_request, user_service_playlists_request,
    ServicePlaylist, ServiceSong, UserServiceData,
};
use crate::lib::security::code_generator;
use crate::lib::util::{
    format_url, get_service_token_cookies, open_link, set_service_token_cookie, remove_service_token_cookie, service_icon_html
};
use crate::routes::guild::DefaultProps;
use gloo::net::http::Request;
use gloo::timers::callback::Timeout;
use pkce::{code_challenge, code_verifier};
use serde_json::Value;
use std::collections::HashMap;
use std::str::from_utf8;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement, MouseEvent};
use yew::html::onscroll::Event;
use yew::platform::spawn_local;
use yew::{function_component, html, use_context, use_mut_ref, Callback, Html, UseStateHandle};

#[function_component(PlaylistsAdd)]
pub fn playlists_add() -> Html {
    let default_props = use_context::<DefaultProps>().expect("ctx");
    let playlists_index = default_props.service_playlists_index;
    let active_services = default_props.active_services;
    let selected_service = default_props.selected_service;
    let user_service_data = default_props.user_service_data;
    let modal_attributes_state = default_props.modal_attributes;
    let service_error_modal = ModalAttributes {
        title: String::from("Login Error"),
        content: html! {{"There was an error with the login process."}},
        modal_type: ModalType::Notification,
        color: ModalColor::Danger,
        confirm: {
            let modal_attributes_state = modal_attributes_state.clone();
            Callback::from(move |_| {
                modal_attributes_state.set(ModalAttributes {
                    open: false,
                    ..Default::default()
                })
            })
        },
        ..Default::default()
    }; // Add logic to turn on modal instead of just attributes (Simplifies login callbacks)
    let spotify_login = {
        let modal_attributes_state = modal_attributes_state.clone();
        let active_services = active_services.clone();
        let selected_service = selected_service.clone();
        Callback::from(move |_: MouseEvent| {
            let modal_attributes_state = modal_attributes_state.clone();
            let service_error_modal = service_error_modal.clone();
            let active_services = active_services.clone();
            let selected_service = selected_service.clone();

            let state = code_generator(16);
            let verifier = code_verifier(64);
            let challenge = code_challenge(&verifier);
            let origin = window().expect("window").location().origin().unwrap();
            open_link(format!("\
                https://accounts.spotify.com/authorize\
                ?response_type=code\
                &client_id=e5757cf7de4b41df9f33abc374f15bf2\
                &code_challenge_method=S256\
                &code_challenge={challenge}\
                &state={}\
                &scope=user-read-private user-read-email playlist-read-private\
                &redirect_uri={origin}/api/authorize",state
            ));
            api_link_request(
                state,
                Callback::from(move |code| {
                    if code == *"error" {
                        modal_attributes_state.set(service_error_modal.clone());
                        return;
                    }
                    let origin = origin.clone();
                    let verifier = verifier.clone();
                    let modal_attributes_state = modal_attributes_state.clone();
                    let active_services = active_services.clone();
                    let selected_service = selected_service.clone();
                    spawn_local(async move {
                        let response: HashMap<String, Value> =
                            Request::post(&format_url("https://accounts.spotify.com/api/token"))
                                .header("Content-Type", "application/x-www-form-urlencoded")
                                .body(&format!("\
                                    client_id=e5757cf7de4b41df9f33abc374f15bf2\
                                    &grant_type=authorization_code\
                                    &code={code}\
                                    &redirect_uri={origin}/api/authorize\
                                    &code_verifier={}",from_utf8(&verifier).unwrap()
                                )).unwrap()
                                .send().await.unwrap().json().await.unwrap();
                        set_service_token_cookie(
                            "spotify",
                            response.get("access_token").unwrap().as_str().unwrap(),
                            response.get("expires_in").unwrap().as_i64().unwrap(),
                            response.get("refresh_token").unwrap().as_str().unwrap(),
                        );
                        modal_attributes_state.set(ModalAttributes {
                            open: false,
                            ..Default::default()
                        });
                        active_services.set(get_service_token_cookies().await);
                        selected_service.set("spotify");
                    });
                }),
            );
        })
    };
    let logout = {
        let active_services = active_services.clone();
        let selected_service = selected_service.clone();
        let user_service_data = user_service_data.clone();
        Box::new(move |service| {
            let active_services = active_services.clone();
            let selected_service = selected_service.clone();
            let user_service_data = user_service_data.clone();
            Callback::from(move |_| {
                let active_services = active_services.clone();
                remove_service_token_cookie(service);
                if *selected_service == service {
                    selected_service.set("");
                    user_service_data.set(UserServiceData {
                        selected_playlist: None,
                        playlists: None,
                        songs: None,
                        playlist_count: 0,
                        ..(*user_service_data).clone()
                    })
                }
                spawn_local(async move {
                    active_services.set(get_service_token_cookies().await);
                    
                })
            })
        })
    };
    let add_account_modal = {
        let modal_attributes_state = modal_attributes_state.clone();
        Callback::from(move |_| {
            modal_attributes_state.set(ModalAttributes {
                title: String::from("Add Account"),
                content: html! {
                    <div class="relative flex flex-row gap-[2vw] h-[5vw] text-[2vw]">
                        <button class="flex items-center justify-center bg-neutral-700 p-[1.6vw] rounded h-full aspect-square" onclick={spotify_login.clone()}><i class="fa-brands fa-spotify" /></button>
                        //<button class="flex items-center justify-center bg-neutral-700 p-[1.6vw] rounded h-full aspect-square"><i class="fa-brands fa-youtube" /></button>
                        //<button class="flex items-center justify-center bg-neutral-700 p-[1.6vw] rounded h-full aspect-square"><i class="fa-brands fa-soundcloud" /></button>
                    </div>
                },
                modal_type: ModalType::Selection,
                color: ModalColor::Primary,
                ..Default::default()
            })
        })
    };
    let select_playlist = {
        let active_services = active_services.clone();
        let selected_service = *selected_service;
        let user_service_data = user_service_data.clone();
        Box::new(move |playlist: ServicePlaylist| {
            let active_services = active_services.clone();
            let user_service_data = user_service_data.clone();
            Callback::from(move |_| {
                user_service_data.set(UserServiceData {
                    selected_playlist: Some(playlist.clone()),
                    ..(*user_service_data).clone()
                });
                let set_songs = {
                    let playlist = playlist.clone();
                    let user_service_data = user_service_data.clone();
                    Callback::from(move |songs: Vec<ServiceSong>| {
                        user_service_data.set(UserServiceData {
                            selected_playlist: Some(playlist.clone()),
                            songs: Some(songs),
                            ..(*user_service_data).clone()
                        })
                    })
                };
                user_service_playlist_songs_request(
                    active_services.clone(),
                    selected_service,
                    set_songs,
                    playlist.clone(),
                    user_service_data.songs.clone().unwrap_or_default(),
                    false,
                )
            })
        })
    };
    let load_more_songs_debounce = use_mut_ref(|| None);
    let load_more_songs = {
        let active_services = active_services.clone();
        let selected_service = *selected_service;
        let user_service_data = user_service_data.clone();
        Callback::from(move |event: Event| {
            if user_service_data.songs.clone().unwrap().len() as u64
                != user_service_data
                    .selected_playlist
                    .clone()
                    .unwrap()
                    .song_count
                    .unwrap()
            {
                let element = event.target().unwrap().dyn_into::<HtmlElement>().unwrap();
                if element.offset_height() + element.scroll_top() + 1 >= element.scroll_height() {
                    let active_services = active_services.clone();
                    let user_service_data = user_service_data.clone();
                    *load_more_songs_debounce.borrow_mut() = Some(Timeout::new(500, move || {
                        user_service_playlist_songs_request(
                            active_services.clone(),
                            selected_service,
                            {
                                let user_service_data = user_service_data.clone();
                                Callback::from(move |songs| {
                                    element.scroll_to_with_x_and_y(
                                        0.0,
                                        (element.scroll_height() - element.offset_height() - 20)
                                            as f64,
                                    );
                                    user_service_data.set(UserServiceData {
                                        songs: Some(songs),
                                        ..(*user_service_data).clone()
                                    })
                                })
                            },
                            user_service_data.selected_playlist.clone().unwrap(),
                            user_service_data.songs.clone().unwrap_or_default(),
                            false,
                        )
                    }));
                }
            }
        })
    };
    let load_all_songs = {
        let active_services = active_services.clone();
        let selected_service = *selected_service;
        let user_service_data = user_service_data.clone();
        Callback::from(
            move |(playlist, data_callback): (ServicePlaylist, Callback<Vec<ServiceSong>>)| {
                let set_songs = {
                    let playlist = playlist.clone();
                    let user_service_data = user_service_data.clone();
                    Callback::from(move |songs: Vec<ServiceSong>| {
                        user_service_data.set(UserServiceData {
                            selected_playlist: Some(playlist.clone()),
                            songs: Some(songs.clone()),
                            ..(*user_service_data).clone()
                        });
                        data_callback.emit(songs.clone())
                    })
                };
                user_service_playlist_songs_request(
                    active_services.clone(),
                    selected_service,
                    set_songs,
                    playlist,
                    user_service_data.songs.clone().unwrap_or_default(),
                    true,
                );
            },
        )
    };
    // Three functions below can be combined to one
    let next_page = {
        let active_services = active_services.clone();
        let selected_service = *selected_service;
        let user_service_data = user_service_data.clone();
        let playlists_index = playlists_index.clone();
        Callback::from(move |_| {
            user_service_playlists_request(
                active_services.clone(),
                selected_service,
                user_service_data.clone(),
                *playlists_index + 1,
            );
            playlists_index.set(*playlists_index + 1);
        })
    };
    let prev_page = {
        let active_services = active_services.clone();
        let selected_service = *selected_service;
        let user_service_data = user_service_data.clone();
        let playlists_index = playlists_index.clone();
        Callback::from(move |_| {
            user_service_playlists_request(
                active_services.clone(),
                selected_service,
                user_service_data.clone(),
                *playlists_index - 1,
            );
            playlists_index.set(*playlists_index - 1);
        })
    };
    let close_playlist_info = {
        let user_service_data = user_service_data.clone();
        Callback::from(move |_| {
            user_service_data.set(UserServiceData {
                selected_playlist: None,
                songs: None,
                ..(*user_service_data).clone()
            })
        })
    };
    html! {
        <div class="h-full flex flex-row divide-x divide-x-[0.2vw] divide-neutral-700">
            <div class="w-full h-full flex flex-col pr-[0.5vw]">
                if user_service_data.selected_playlist.is_none() {
                    <div class="w-full flex flex-row items-center text-[1.5vw] pb-[1vw]">
                        <img class="rounded-full h-[3vw] aspect-square" src={format_url(user_service_data.avatar.as_str())} />
                        <span class="ml-[0.6vw] font-semibold">{&user_service_data.name}</span>
                        <div class="w-full flex flex-row justify-end">
                                <span class="mr-[0.6vw]">{format!("Page {} / {}", *playlists_index, (user_service_data.playlist_count / 8) + 1)}</span>
                                if *playlists_index > 1 {
                                    <button onclick={prev_page}><i class="fa-solid fa-chevron-left" /></button>
                                }
                                if *playlists_index < ((user_service_data.playlist_count / 8) + 1) as usize {
                                    <button onclick={next_page}><i class="fa-solid fa-chevron-right" /></button>
                                }
                        </div>
                    </div>
                    <div class="relative w-full h-fit grid grid-cols-4 justify-items-center gap-[2vh] pb-[1vw]">
                        {for user_service_data.playlists.clone().unwrap_or_default().iter().map(|playlist| {
                            html! {
                                <button class="relative flex flex-col w-[26vh] max-w-full hover:-translate-y-[1vw] transition-transform duration-150 group" onclick={select_playlist(playlist.clone()).clone()}>
                                    <span class="absolute bg-neutral-900 text-[1.6vh] rounded m-[0.6vh] p-[0.6vh] top-0 z-10">{format!("{} Songs", playlist.song_count.unwrap())}</span>
                                    if playlist.cover.is_some() {
                                        <img class="rounded-t-md w-full aspect-square" src={format_url(playlist.cover.clone().unwrap().as_str())} />
                                    } else {
                                        <div class="bg-neutral-950 w-full aspect-square rounded-t-md"></div>
                                    }
                                    <p class="bg-neutral-800 px-[2vh] py-[1.2vh] w-full text-[2vh] group-hover:text-sky-500 font-semibold text-center rounded-b-md duration-150">{&playlist.title}</p>
                                </button>
                            }
                        })}
                    </div>
                } else {
                    <GenericPlaylistComponent
                        selected_playlist={GenericPlaylist::from((*user_service_data).clone().selected_playlist.unwrap())}
                        songs={user_service_data.songs.clone().map(|songs| songs.into_iter().map(GenericSong::from).collect::<Vec<GenericSong>>()) }
                        update={load_more_songs}
                        load_all={load_all_songs}
                        close={close_playlist_info}
                    />
                }
            </div>
            <div class="w-[25vw] pl-[0.5vw] h-full flex flex-col items-center">
                { for active_services.iter().map(|(&service, _)| {
                    let mut service_string: Vec<char> = service.chars().collect();
                    service_string[0] = service_string[0].to_uppercase().nth(0).unwrap();
                    html! {
                        if active_services.get(service).unwrap().is_some() {
                            <button class={format!("w-full relative text-[2vw] py-[0.5vw] font-semibold {}", if *selected_service == service {"bg-neutral-800 border-r-2 border-sky-400"} else {""})} onclick={let selected_service = selected_service.clone(); Callback::from(move |_| selected_service.set(service))}>
                                {service_icon_html(service.to_string())}
                                <span class="ml-[0.4vw]">{service_string.into_iter().collect::<String>()}</span>
                                <button class="absolute right-0 mr-[0.5vw] hover:text-red-500 duration-150" onclick={logout(service)}><i class="fa-solid fa-right-from-bracket" /></button>
                            </button>
                        }
                    }
                })}
                /*
                if active_services.get("spotify").unwrap().is_some() {
                    <button class={format!("w-full text-[2vw] py-[0.5vw] font-semibold {}", if *selected_service == "spotify" {"bg-neutral-800 border-r-2 border-sky-400"} else {""})} onclick={let selected_service = selected_service.clone(); Callback::from(move |_| selected_service.set("spotify"))}>
                        <i class="fa-brands fa-spotify mr-[0.4vw]"/>
                        {"Spotify"}
                    </button>
                }
                <button class={format!("w-full text-[2vw] py-[0.5vw] font-semibold {}", if *selected_service == "youtube" {"bg-neutral-800 border-r-2 border-sky-400"} else {""})} onclick={let selected_service = selected_service.clone(); Callback::from(move |_| selected_service.set("youtube"))}>
                    <i class="fa-brands fa-youtube mr-[0.4vw]"/>
                    {"YouTube"}
                </button>

                 */
                <button class="text-[1.5vw] mt-[1vw] p-[1vw] bg-neutral-800 rounded" onclick={add_account_modal}>{"Add Account"}</button>
            </div>
        </div>
    }
}
