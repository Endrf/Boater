use web_sys::window;
use yew::{Callback, function_component, html, Html, use_force_update};

#[function_component(SettingsAppearance)]
pub fn appearance() -> Html {
    let update = use_force_update();
    let storage = window().unwrap().local_storage().unwrap().unwrap();
    let enabled_button = "bg-sky-950 text-sky-500";
    html! {
        <div class="h-fit flex flex-row flex-wrap text-[1.4vw]">
            <div class="w-full text-[2vw] font-semibold">
                {"UI Preferences"}
            </div>
            // Create ui abstraction for individual settings
            <div class="flex flex-row justify-between items-center w-1/2 p-[0.5vw]">
                {"Main Sidebar"}
                <div class="bg-neutral-800 rounded-md text-[1vw] font-semibold">
                    <button
                        class={format!("p-[0.5vw] rounded-l-md {}", if storage.get("sidebar-hidden").unwrap().is_none() {enabled_button} else {""})}
                        onclick={
                            let update = update.clone();
                            let storage = storage.clone();
                            Callback::from(move |_| {storage.remove_item("sidebar-hidden").unwrap(); update.force_update()})}
                    >{"Open"}</button>
                    <button
                        class={format!("p-[0.5vw] rounded-r-md {}", if storage.get("sidebar-hidden").unwrap().is_some() {enabled_button} else {""})}
                        onclick={
                            let update = update.clone();
                            let storage = storage.clone();
                            Callback::from(move |_| {storage.set_item("sidebar-hidden", "true").unwrap(); update.force_update()})
                        }
                    >{"Closed"}</button>
                </div>
            </div>
        </div>
    }
}
