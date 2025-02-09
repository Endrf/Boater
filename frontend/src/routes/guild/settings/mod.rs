mod status;
mod appearance;

use yew::{Callback, function_component, Html, html, use_state};
use crate::routes::guild::settings::{appearance::SettingsAppearance, status::SettingsStatus};

#[function_component(Settings)]
pub fn settings() -> Html {
    let section = use_state(|| 0);
    let nav_active = "bg-neutral-900 decoration-sky-400 text-sky-500";
    html! {
        <div class="flex justify-center w-full h-full">
            <div class="absolute left-[20vw] m-[1vw] w-fit flex flex-row items-center h-[3.6vw] text-[1vw] font-semibold bg-neutral-800 rounded-lg">
                <button class={format!("h-[3.6vw] underline-offset-4 p-[1vw] decoration-wavy decoration-1 rounded-l-lg {}", if *section == 0 {nav_active} else {"hover:text-sky-500"})} onclick={let section = section.clone(); Callback::from(move |_| section.set(0))}>
                    <i class="fa-solid fa-server mr-[0.6vw]" />
                    {"Status"}
                </button>
                <button class={format!("h-[3.6vw] underline-offset-4 p-[1vw] decoration-wavy decoration-1 {}", if *section == 1 {nav_active} else {"hover:text-sky-500"})} onclick={let section = section.clone(); Callback::from(move |_| section.set(1))}>
                    <i class="fa-solid fa-wand-magic-sparkles mr-[0.6vw]" />
                    {"Appearance"}
                </button>
                <button class={format!("h-[3.6vw] underline-offset-4 p-[1vw] decoration-wavy decoration-1 {}", if *section == 2 {nav_active} else {"hover:text-sky-500"})} onclick={let section = section.clone(); Callback::from(move |_| section.set(2))}>
                    <i class="fa-solid fa-circle-play mr-[0.6vw]" />
                    {"Audio Services"}
                </button>
                <button class={format!("h-[3.6vw] underline-offset-4 p-[1vw] decoration-wavy decoration-1 rounded-r-lg {}", if *section == 3 {nav_active} else {"hover:text-sky-500"})} onclick={let section = section.clone(); Callback::from(move |_| section.set(3))}>
                    <i class="fa-solid fa-circle-question mr-[0.6vw]" />
                    {"Info"}
                </button>
            </div>
            <div class="w-[80vw] h-full p-[1vw] pt-[5.6vw]">
                <div class="w-full h-full p-[1vw] bg-neutral-900 rounded-xl">
                    if *section == 0 {
                        <SettingsStatus />
                    } else if *section == 1 {
                        <SettingsAppearance />
                    } else {
                        <div class="h-full w-full flex justify-center items-center">{"Placeholder"}</div>
                    }
                </div>
            </div>
        </div>
    }
}
