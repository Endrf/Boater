use yew::{function_component, Html, html};
use yew_router::prelude::Redirect;
use yew::prelude::*;
use web_sys::window;
use crate::{getGuildId, Route};
use crate::lib::util::format_url;

#[function_component(Home)]
pub fn home() -> Html {
    let guild_id = getGuildId();
    
    html! {
        <Redirect<Route> to={Route::Guild {id: guild_id}} />
    }
}
