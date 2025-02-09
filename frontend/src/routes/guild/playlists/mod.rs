pub(crate) mod add;
mod page;
mod saved;
mod search;

use crate::routes::guild::playlists::add::PlaylistsAdd;
use crate::routes::guild::playlists::page::PlaylistsHome;
use crate::routes::guild::playlists::saved::PlaylistsSaved;
use crate::routes::guild::playlists::search::PlaylistsSearch;
use crate::routes::guild::DefaultProps;
use yew::{function_component, html, use_context, Callback, Html};

#[function_component(Playlists)]
pub fn playlists() -> Html {
    let section = use_context::<DefaultProps>()
        .expect("ctx")
        .service_playlists_section;
    let nav_active = "bg-neutral-900 decoration-sky-400 text-sky-500";
    html! {
        <div class="flex justify-center w-full h-full">
            <div class="absolute left-[20vw] m-[1vw] w-fit flex flex-row items-center h-[3.6vw] text-[1vw] font-semibold bg-neutral-800 rounded-lg">
                <button class={format!("h-[3.6vw] underline-offset-4 p-[1vw] decoration-wavy decoration-1 rounded-l-lg {}", if *section == 0 {nav_active} else {"hover:text-sky-500"})} onclick={let section = section.clone(); Callback::from(move |_| section.set(0))}>
                    <i class="fa-solid fa-home mr-[0.6vw]" />
                    {"Home"}
                </button>
                <button class={format!("h-[3.6vw] underline-offset-4 p-[1vw] decoration-wavy decoration-1 {}", if *section == 1 {nav_active} else {"hover:text-sky-500"})} onclick={let section = section.clone(); Callback::from(move |_| section.set(1))}>
                    <i class="fa-solid fa-cloud mr-[0.6vw]" />
                    {"Saved"}
                </button>
                <button class={format!("h-[3.6vw] underline-offset-4 p-[1vw] decoration-wavy decoration-1 {}", if *section == 2 {nav_active} else {"hover:text-sky-500"})} onclick={let section = section.clone(); Callback::from(move |_| section.set(2))}>
                    <i class="fa-solid fa-plus mr-[0.6vw]" />
                    {"Add From Account"}
                </button>
                <button class={format!("h-[3.6vw] underline-offset-4 p-[1vw] decoration-wavy decoration-1 rounded-r-lg {}", if *section == 3 {nav_active} else {"hover:text-sky-500"})} onclick={let section = section.clone(); Callback::from(move |_| section.set(3))}>
                    <i class="fa-solid fa-magnifying-glass mr-[0.6vw]" />
                    {"Search"}
                </button>
            </div>
            <div class="w-[80vw] h-full p-[1vw] pt-[5.6vw]">
                <div class="w-full h-full p-[1vw] bg-neutral-900 rounded-xl">
                    if *section == 0 {
                        <PlaylistsHome />
                    } else if *section == 1 {
                        <PlaylistsSaved />
                    } else if *section == 2 {
                        <PlaylistsAdd />
                    } else if *section == 3 {
                        <PlaylistsSearch />
                    }
                </div>
            </div>
        </div>
    }
}
