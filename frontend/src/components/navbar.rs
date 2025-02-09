use web_sys::window;
use yew::{Callback, classes, function_component, Html, html, Properties, use_state, use_context};
use yew_router::hooks:: use_route;
use yew_router::prelude::Link;
use yew_router::Routable;
use crate::Route;
use crate::guild::DefaultProps;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) guild_id: String
}

#[function_component(NavBar)]
pub fn navigation(props: &Props) -> Html {
    let default_props = use_context::<DefaultProps>().expect("ctx");
    let route = use_route::<Route>();
    let nav_collapse = use_state(
        || if window().unwrap().local_storage().unwrap().unwrap().get("sidebar-hidden").unwrap().is_some() {
            true
        } else { false }
    );
    let toggle_collapse = {
        let nav_collapse = nav_collapse.clone();
        if *nav_collapse {
            Callback::from(move |_| nav_collapse.set(false))
        } else {
            Callback::from(move |_| nav_collapse.set(true))
        }
    };
    html! {
        <>
            <div class={format!("w-[20vw] left-0 top-0 flex flex-col grow-0 shrink-0 p-[1vw] z-30 {}", if *nav_collapse {"absolute h-fit"} else {"relative h-full bg-neutral-900"})}>
                <div class="h-fit flex items-center"> // Nav Header
                    <div class="relative mr-[0.5vw] h-[3.5vw] aspect-square">
                        if default_props.ws.ready_state() == 1 { <div class="absolute bottom-[0.2vw] right-[0.2vw] p-[0.4vw] bg-green-500 rounded-full border-neutral-900 border-[0.2vw]" /> }
                        else { <div class="absolute bottom-[0.2vw] right-[0.2vw] p-[0.4vw] bg-red-600 rounded-full border-neutral-900 border-[0.2vw]" /> }

                        <img src="/static/Boater.png" alt="Boater Logo" class="h-full aspect-square bg-neutral-900 rounded-full shadow p-[0.2vw]" />
                    </div>
                    <div class={format!("relative w-full h-[3.5vw] flex items-center justify-around text-[1.5vw] rounded-lg shadow {}", if *nav_collapse {"bg-neutral-900"} else {"bg-neutral-950"})}>
                        <Link<Route> classes={classes!(format!("w-full h-full flex items-center justify-center p-[0.6vw] hover:bg-neutral-800 group rounded-l-lg {}", if route.clone().unwrap().to_path() == format!("/guild/{}", props.guild_id.clone()) {"bg-neutral-800"} else {""}))} to={Route::Guild {id: props.guild_id.clone()}}>
                            <i class="fa-solid fa-music" />
                            <span class={format!("tooltip top-[110%] {}", if *nav_collapse {"bg-neutral-900"} else {"bg-neutral-950"})}>{"Music Queue"}</span>
                        </Link<Route>>
                        <Link<Route> classes={classes!(format!("w-full h-full flex items-center justify-center p-[0.6vw] hover:bg-neutral-800 group {}", if route.clone().unwrap().to_path() == format!("/guild/{}/playlists", props.guild_id.clone()) {"bg-neutral-800"} else {""}))} to={Route::Playlists {id: props.guild_id.clone()}}>
                            <i class="fa-solid fa-record-vinyl" />
                            <span class={format!("tooltip top-[110%] {}", if *nav_collapse {"bg-neutral-900"} else {"bg-neutral-950"})}>{"User Playlists"}</span>
                        </Link<Route>>
                        <Link<Route> classes={classes!(format!("w-full h-full flex items-center justify-center p-[0.6vw] hover:bg-neutral-800 group rounded-r-lg {}", if route.clone().unwrap().to_path() == format!("/guild/{}/settings", props.guild_id.clone()) {"bg-neutral-800"} else {""}))} to={Route::Settings {id: props.guild_id.clone()}}>
                            <i class="fa-solid fa-sliders" />
                            <span class={format!("tooltip top-[110%] {}", if *nav_collapse {"bg-neutral-900"} else {"bg-neutral-950"})}>{"Settings"}</span>
                        </Link<Route>>
                    </div>
                </div>
                if !*nav_collapse { // Nav Body
                    <div class="h-full w-full pt-[1vw] flex flex-col gap-[1vw] items-center text-[1vw]">
                        /*
                        <div class="flex flex-col w-full h-full gap-[1vw] text-[1.5vw] bg-neutral-950 rounded-lg"> // Change to liked songs / playlists
                        </div>
                        */
                        <div class="flex justify-center items-center h-full">{"Placeholder"}</div>
                        <div class="justify-self-end">
                            {"Boater-UI (BUI) alpha v0.4.0"}
                        </div>
                    </div>
                }
            </div>
            <div class="relative z-[20] h-full flex items-center"> // Side Controls
                <div class="absolute p-[1vw] -translate-x-[4vw] hover:translate-x-0 transition-transform ease-in-out duration-150">
                    <div class="relative flex flex-col items-center gap-[0.6vw] p-[0.6vw] bg-neutral-900/70 backdrop-blur-lg rounded-lg ring-[0.2vw] ring-sky-400 ring-inset text-[2vw]">
                        <button class="flex items-center group hover:text-sky-400" onclick={toggle_collapse}>
                            if *nav_collapse {
                                <i class="fa-solid fa-angles-right" />
                                <span class="tooltip left-[110%] text-[1.2vw]">{"Reveal Sidebar"}</span>
                            } else {
                                <i class="fa-solid fa-angles-left" />
                                <span class="tooltip left-[110%] text-[1.2vw]">{"Collapse Sidebar"}</span>
                            }
                        </button>

                    </div>
                </div>
            </div>
        </>
    }
}
