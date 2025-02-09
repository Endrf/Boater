mod home;
mod routes {
    pub mod guild;
    pub mod not_found;
}
mod components {
    pub mod generic_playlist;
    pub mod modal;
    pub mod navbar;
    pub mod player;
}
mod lib {
    pub mod api;
    pub mod security;
    pub mod util;
    pub mod websocket;
}

use crate::routes::guild;
use crate::routes::not_found::NotFound;
use guild::Guild;
use home::Home;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(module = "/src/package.js")]
extern "C" {
    pub fn initializeDiscordSDK() -> JsValue;
    pub fn openExternalLink(url: String);
    pub fn getGuildId() -> String;
    pub fn getChannelId() -> String;
    pub fn getUserId() -> Option<String>;
    pub fn getUserName() -> String;
    pub fn getDisplayName() -> String;
    pub fn getUserAvatar() -> String;
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/guild/:id")]
    Guild { id: String },
    #[at("/guild/:id/playlists")]
    Playlists { id: String },
    #[at("/guild/:id/settings")]
    Settings { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Guild { id } => html! { <Guild guild_id={id} /> },
        Route::Playlists { id } => html! { <Guild guild_id={id} /> },
        Route::Settings { id } => html! { <Guild guild_id={id} /> },
        Route::NotFound => html! { <NotFound /> },
    }
}

#[function_component(Main)]
fn app() -> Html {
    initializeDiscordSDK();
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Main>::new().render();
}
