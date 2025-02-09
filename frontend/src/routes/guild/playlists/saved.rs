use crate::components::generic_playlist::{GenericPlaylist, GenericPlaylistComponent, GenericSong};
use crate::getUserId;
use crate::lib::util::{format_url, service_icon_html};
use crate::lib::websocket::{
    get_boater_playlist_songs, get_boater_user_playlists, get_boater_users, BoaterSong, BoaterUser,
    BoaterUserPlaylist,
};
use crate::routes::guild::DefaultProps;
use serde_json::from_value;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::html::onchange::Event as ChangeEvent;
use yew::html::onclick::Event as ClickEvent;
use yew::html::oninput::Event;
use yew::{function_component, html, use_context, use_effect_with, use_state, UseStateHandle, Callback, Html};

#[function_component(PlaylistsSaved)]
pub fn playlists_saved() -> Html {
    let default_props = use_context::<DefaultProps>().expect("context");
    let selected_boater_user = default_props.selected_boater_user.clone();
    let selected_boater_playlist = default_props.selected_boater_playlist.clone();
    let data = (*default_props.db_data).clone();
    let users = if let Some(db_users) = data.get("users") {
        from_value::<Vec<BoaterUser>>(db_users.clone()).unwrap()
    } else {
        Vec::new()
    };
    let selected_user_data = if let Some(id) = (*selected_boater_user).clone() {
        users.iter().find(|&user| user.id == id)
    } else {
        None
    };
    let all_playlists: Vec<BoaterUserPlaylist> = if let Some(db_user_playlists) = data.get("userPlaylists") {
        from_value::<Vec<BoaterUserPlaylist>>(db_user_playlists.clone()).unwrap()
    } else {
        Vec::new()
    };
    let visible_playlists = use_state(Vec::<BoaterUserPlaylist>::new);
    let selected_playlist = if (*selected_boater_playlist).is_some() {
        if let Some(playlist) = all_playlists.clone().into_iter().find(|playlist| playlist.id == (*selected_boater_playlist).clone().unwrap()) {
            playlist
        } else {
            selected_boater_playlist.set(None);
            Default::default()
        }
    } else {
        Default::default()
    };
    let playlist_songs = data
        .get("playlistSongs")
        .map(|playlist_songs| from_value::<Vec<BoaterSong>>(playlist_songs.clone()).unwrap());
    let load_user_playlists = {
        let ws = (*default_props.ws).clone();
        Box::new(move |id: String| {
            let ws = ws.clone();
            Callback::from(move |_| {
                get_boater_user_playlists(ws.clone(), id.clone())
            })
        })
    };
    let close_playlist = {
        let selected_boater_playlist = selected_boater_playlist.clone();
        let data = data.clone();
        let db_data = default_props.db_data.clone();
        Callback::from(move |_| {
            selected_boater_playlist.set(None);
            let mut new_data = data.clone();
            new_data.remove("playlistSongs");
            db_data.set(new_data)
        })
    };

    let search_value = use_state(String::new);
    let sort_type = use_state(|| String::from("new_old"));
    use_effect_with((), {
        let ws = (*default_props.ws).clone();
        let selected_boater_user = selected_boater_user.clone();
        let selected_boater_playlist = selected_boater_playlist.clone();
        let load_user_playlists = load_user_playlists.clone();
        move |_| {
            if selected_boater_playlist.is_none() {
                get_boater_users(ws.clone());
            }
            let id = getUserId().unwrap_or_default();
            selected_boater_user.set(Some(id.clone()));
            load_user_playlists(id).clone().emit(ClickEvent::new("").unwrap());
        }
    });
    use_effect_with(((*search_value).clone(), (*sort_type).clone()), {
        let visible_playlists = visible_playlists.clone();
        let all_playlists = all_playlists.clone();
        move |(search_value, sort_type)| {
            let mut modified_playlists = all_playlists.clone();
            match sort_type.as_str() {
                "new_old" => modified_playlists = modified_playlists.into_iter().rev().collect(),
                "old_new" => {}
                "a_z" => modified_playlists.sort_by(|a, b| a.title.cmp(&b.title)),
                "z_a" => modified_playlists.sort_by(|a, b| b.title.cmp(&a.title)),
                _ => {}
            }
            if !search_value.as_str().is_empty() {
                modified_playlists.retain(|playlist| {
                    playlist
                        .title
                        .to_lowercase()
                        .contains(search_value.as_str())
                })
            }
            visible_playlists.set(modified_playlists)
        }
    });
    use_effect_with(all_playlists.clone(), {
        let visible_playlists = visible_playlists.clone();
        move |data: &Vec<BoaterUserPlaylist>| visible_playlists.set(data.clone().into_iter().rev().collect())
    });
    html! {
        <div class="flex flex-row h-full gap-[1vw]">
            if default_props.selected_boater_playlist.is_none() {
                <div class="shrink-0 grow-0 bg-neutral-800 flex flex-col rounded-md p-[0.5vw] w-fit h-fit">
                    { for users.iter().map(|user| {
                        html! {
                            <div class={format!("{}", if user.id == getUserId().unwrap_or_default() {"order-first"} else {""})}>
                                <button class={format!("p-[0.2vw] rounded-md {}", if let Some(id) = (*selected_boater_user).clone() {if id == user.id {"bg-neutral-900"} else {""}} else {""})}  onclick={load_user_playlists(user.id.clone())}>
                                    <img class={"w-[3vw] aspect-square rounded-full"} src={format_url(&user.avatar)} />
                                </button>
                                if user.id == getUserId().unwrap_or_default() {<hr class="h-[0.2vw] mb-[0.2vw] bg-neutral-700 border-0" />}
                            </div>
                        }
                    })}
                </div>
                if selected_user_data.is_some() {
                    <div class="flex flex-col w-full gap-[1vw]">
                        <div class="flex flex-row w-full overflow-hidden gap-[1vw] h-[3.4vw]">
                            <span class="flex w-3/4 justify-center text-[2vw] font-semibold">{format!("{}'s Playlists", selected_user_data.unwrap().display_name.clone())}</span>
                            <div class="w-1/4 h-full">
                                if selected_user_data.unwrap().id == getUserId().unwrap_or_default() {
                                    <button class="bg-neutral-800 rounded-md w-full h-full text-[1.4vw]">
                                        <i class="fa-solid fa-plus mr-[0.5vw]" />
                                        <span class="font-semibold">{"Create Playlist"}</span>
                                    </button>
                                }
                            </div>
                        </div>
                        <div class="flex flex-row w-full h-full overflow-hidden gap-[1vw]">
                            <div class="relative flex flex-col w-3/4 gap-[0.5vw] overflow-y-auto snap-y">
                                { for visible_playlists.iter().map(|playlist| {
                                    html! {
                                        <button class="w-full flex flex-row snap-start text-[1vw]" style="height: calc((100% - 1.5vw) / 4);" onclick={let playlist = playlist.clone(); let selected_boater_playlist = selected_boater_playlist.clone(); let ws = (*default_props.ws).clone(); Callback::from(move |_| {selected_boater_playlist.set(Some(playlist.id.clone())); get_boater_playlist_songs(ws.clone(), playlist.id.clone(), playlist.owner.clone())})}>
                                            <div class="relative h-full aspect-square rounded-l bg-neutral-950">
                                                <div class="absolute top-[0.2vw] left-[0.2vw] py-[0.2vw] px-[0.4vw] text-[1vw] rounded-md bg-neutral-800/70 backdrop-blur z-[1]">
                                                    {service_icon_html(playlist.provider.clone())}
                                                </div>
                                                <div class="absolute bottom-[0.2vw] right-[0.2vw] py-[0.2vw] px-[0.4vw] text-[1vw] rounded-md bg-neutral-800/70 backdrop-blur z-[1]">
                                                    {format!("{} Songs", &playlist.song_count)}
                                                </div>
                                                <img class="rounded-l" width="256" height="256" src={format_url(playlist.cover.clone().unwrap_or_default().as_str())} />
                                            </div>
                                            <div class="bg-neutral-800 flex flex-col rounded-r p-[0.5vw] w-full h-full text-[1.2vw]">
                                                <div class="flex flex-col text-left w-full h-full">
                                                    <span class="text-[1.5vw]">{&playlist.title}</span>
                                                    <span class="text-neutral-400">{"By "}{&playlist.artist}</span>
                                                </div>
                                                <div class="w-fit self-end">
                                                    <button class="bg-neutral-900 p-[0.4vw] rounded-md"><i class="fa-solid fa-thumbs-up" /></button>
                                                    <button class="bg-neutral-900 p-[0.4vw] rounded-md"><i class="fa-solid fa-play" /></button>
                                                </div>
                                            </div>
                                        </button>
                                    }
                                }) }
                            </div>
                            <div class="bg-neutral-800 flex flex-col items-center p-[1vw] rounded-md w-1/4 h-full">
                                <div class="flex flex-col items-center text-[1vw] w-full gap-[0.5vw]">
                                    <form class="w-full">
                                        <span class="text-[1.4vw] font-semibold">{"Search"}</span>
                                        <input class="text-center py-[0.5vw] w-full bg-neutral-900 rounded-md appearance-none" placeholder="Search User's Playlists" oninput={let search_value = search_value.clone(); Callback::from(move |e: Event| search_value.set(e.target().expect("value").unchecked_into::<HtmlInputElement>().value().to_lowercase()))} />
                                    </form>
                                    <form class="w-full">
                                        <span class="text-[1.4vw] font-semibold">{"Sort"}</span>
                                        <select class="text-center py-[0.5vw] w-full bg-neutral-900 rounded-md appearance-none" onchange={let sort_type = sort_type.clone(); Callback::from(move |e: ChangeEvent| sort_type.set(e.target().expect("selection").unchecked_into::<HtmlInputElement>().value()))}>
                                            <option class="bg-neutral-900" value="new_old" selected=true>{"Newest-Oldest"}</option>
                                            <option class="bg-neutral-900" value="old_new">{"Oldest-Newest"}</option>
                                            <option class="bg-neutral-900" value="a_z">{"A-Z"}</option>
                                            <option class="bg-neutral-900" value="z_a">{"Z-A"}</option>
                                        </select>
                                    </form>
                                    <form class="w-full">
                                        <span class="text-[1.4vw] font-semibold">{"Filter"}</span>
                                    </form>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            } else {
                <GenericPlaylistComponent
                    selected_playlist={GenericPlaylist::from(selected_playlist)}
                    songs={playlist_songs.map(|songs| songs.into_iter().map(Into::into).collect::<Vec<GenericSong>>())}
                    update={None::<Callback<_>>}
                    load_all={None::<Callback<_>>}
                    close={close_playlist}
                />
            }
        </div>
    }
}
