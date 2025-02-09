use yew::{function_component, Html, html};

#[function_component(PlaylistsHome)]
pub fn playlists_home() -> Html {
    html! {
        <div class="w-full h-full flex flex-col">
            /*
            <div class="font-bold text-[3vw]">{"Most Liked"}</div>
            <div class="bg-neutral-950 rounded-lg w-full h-[13vw]"></div>
            <div class="font-bold text-[3vw]">{"Most Played"}</div>
            <div class="bg-neutral-950 rounded-lg w-full h-[13vw]"></div>
            */
            <div class="h-full w-full flex justify-center items-center">{"Placeholder"}</div>
        </div>
    }
}
