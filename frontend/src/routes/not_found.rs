use yew::{function_component, Html, html};

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <h1>{"Error: 404"}</h1>
    }
}