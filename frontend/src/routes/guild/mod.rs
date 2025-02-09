mod page;
pub(crate) mod playlists;
mod settings;

use crate::components::modal::{Modal, ModalAttributes, ModalType, ModalColor};
use crate::components::navbar::NavBar;
use crate::home::Home;
use crate::lib::api::{user_service_playlists_request, UserServiceData};
use crate::lib::util::{get_service_token_cookies, format_url};
use crate::lib::websocket::WebSocketPayload;
use crate::routes::guild::page::GuildHome;
use crate::routes::guild::settings::Settings;
use crate::routes::not_found::NotFound;
use crate::Route;
use std::{rc::Rc, cell::RefCell};
use gloo::console::info;
use playlists::Playlists;
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{window, MessageEvent, WebSocket};
use web_sys::js_sys::Function;
use yew::platform::{spawn_local, time::sleep};
use yew::{
    function_component, html, use_effect, use_effect_with, use_mut_ref, use_state, Callback,
    ContextProvider, Html, Properties, UseStateHandle,
};
use yew_router::Switch;

#[derive(Clone, Debug, PartialEq)]
pub struct DefaultProps {
    pub ws: UseStateHandle<WebSocket>,
    pub queue_data: UseStateHandle<HashMap<String, Value>>,
    pub db_data: UseStateHandle<HashMap<String, Value>>,
    pub db_response_callback: UseStateHandle<Callback<Value>>,
    pub modal_attributes: UseStateHandle<ModalAttributes>,
    pub service_playlists_section: UseStateHandle<usize>,
    pub service_playlists_index: UseStateHandle<usize>,
    pub active_services: UseStateHandle<HashMap<&'static str, Option<String>>>,
    pub selected_service: UseStateHandle<&'static str>,
    pub selected_boater_user: UseStateHandle<Option<String>>,
    pub selected_boater_playlist: UseStateHandle<Option<String>>,
    pub user_service_data: UseStateHandle<UserServiceData>,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Guild { id: _ } => html! { <GuildHome /> },
        Route::Playlists { id: _ } => html! { <Playlists /> },
        Route::Settings { id: _ } => html! { <Settings /> },
        Route::Home => html! { <Home /> },
        Route::NotFound => html! { <NotFound /> },
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub(crate) guild_id: String,
}

#[function_component(Guild)]
pub fn guild(props: &Props) -> Html {
    info!("redraw");
    let hostname = window().expect_throw("no window").location().hostname().expect_throw("no hostname");
    let mounted = use_mut_ref(|| false);
    let default_props = DefaultProps {
        ws: use_state(|| {
            WebSocket::new(&*("wss://".to_owned() + &hostname + "/.proxy/ws/guild")).unwrap()
        }),
        queue_data: use_state(HashMap::<String, Value>::new),
        db_data: use_state(HashMap::<String, Value>::new),
        db_response_callback: use_state(Default::default),
        modal_attributes: use_state(|| ModalAttributes {
            open: false,
            ..Default::default()
        }),
        service_playlists_section: use_state(|| 0),
        service_playlists_index: use_state(|| 1),
        active_services: use_state(HashMap::<&str, Option<String>>::new),
        selected_service: use_state(Default::default),
        selected_boater_user: use_state(|| None),
        selected_boater_playlist: use_state(|| None),
        user_service_data: use_state(Default::default),
    };
    let reconnect_attempts = use_state(|| 0);
    use_effect({
        let default_props = default_props.clone();
        let props = props.clone();
        let reconnect_attempts = reconnect_attempts.clone();
        move || {
            let default_props = default_props.clone();
            let ws = default_props.ws.clone();
            
            let onopen = {
                let ws = ws.clone();
                let guild_id = props.guild_id.clone();
                Closure::wrap(Box::new(move || {
                    ws.send_with_str(
                        serde_json::to_string(&WebSocketPayload {
                            action: "setGuild",
                            data: Some(guild_id.clone()),
                            ..Default::default()
                        })
                        .unwrap()
                        .as_str(),
                    )
                    .unwrap()
                }) as Box<dyn FnMut()>)
            };
            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();

            let onclose = {
                let default_props = default_props.clone();
                let ws = ws.clone();
                let reconnect_attempts = reconnect_attempts.clone();
                Closure::wrap(Box::new(move || {
                    let default_props = default_props.clone();
                    let ws = ws.clone();
                    let modal_attributes = default_props.modal_attributes.clone();
                    let hostname = window().expect_throw("no window").location().hostname().expect_throw("no hostname");
                    let reconnect_attempts = reconnect_attempts.clone();
                    let allowed_attempts = 5;
                    let mut time = 10;
                    spawn_local(async move {
                        if ws.ready_state() == 3 && *reconnect_attempts == 0 {
                            reconnect_attempts.set(1);
                            ws.set(WebSocket::new(&*("wss://".to_owned() + &hostname + "/.proxy/ws/guild")).unwrap());
                            return;
                        } else if ws.ready_state() == 3 && *reconnect_attempts <= allowed_attempts {
                            while time != 0 {
                                sleep(std::time::Duration::from_secs(1)).await;
                                modal_attributes.set(ModalAttributes {
                                    title: String::from("Websocket Error"),
                                    content: html! {
                                        <div>
                                            {"Attempt: "}{*reconnect_attempts}{" / "}{allowed_attempts}<br />
                                            {"Websocket failed to connect"}<br />{"Retrying in "}{time}{" seconds"}
                                        </div>
                                    },
                                    modal_type: ModalType::Loading,
                                    color: ModalColor::Danger,
                                    ..Default::default()
                                });
                                time -= 1;
                            }
                            modal_attributes.set(ModalAttributes {
                                title: String::from("Websocket Error"),
                                content: html! {
                                    <div>
                                        {"Reconnecting to websocket..."}
                                    </div>
                                },
                                modal_type: ModalType::Loading,
                                color: ModalColor::Danger,
                                ..Default::default()
                            });
                            reconnect_attempts.set(*reconnect_attempts + 1);
                            ws.set(WebSocket::new(&*("wss://".to_owned() + &hostname + "/.proxy/ws/guild")).unwrap());
                        } else {
                            modal_attributes.set(ModalAttributes {
                                title: String::from("Websocket Error"),
                                content: html! {
                                    <div>
                                        {"Could not connect to the websocket"}<br />
                                        {"Is the bot online?"}<br />
                                        <button
                                            class="rounded bg-green-900 hover:bg-green-800 p-[0.6vw] transition-colors duration-150"
                                            onclick={Callback::from(move |_| {
                                                reconnect_attempts.set(0);
                                                ws.set(WebSocket::new(&*("wss://".to_owned() + &hostname + "/.proxy/ws/guild")).unwrap());
                                            })}
                                        >{"Try Again"}</button>
                                    </div>
                                },
                                modal_type: ModalType::Selection,
                                color: ModalColor::Danger,
                                ..Default::default()
                            })
                        }
                    })
                }) as Box<dyn FnMut()>)
            };
            ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
            onclose.forget();

            let onmessage = {
                let ws = ws.clone();
                let modal_attributes = default_props.modal_attributes.clone();
                let queue_data = default_props.queue_data.clone();
                let db_data = default_props.db_data.clone();
                let db_response_callback = default_props.db_response_callback.clone();
                Closure::wrap(Box::new(move |e: MessageEvent| {
                    let Ok(message_data) = e.data().dyn_into::<JsValue>() else {
                        return;
                    };
                    if message_data.as_string().unwrap().contains("[reconnect]") {
                        ws.set_onclose(None);
                        ws.close().unwrap();
                        ws.set(WebSocket::new(message_data.as_string().unwrap().split(' ').last().unwrap()).unwrap());
                        return;
                    } else if message_data.as_string().unwrap().contains("[connected]") {
                        reconnect_attempts.set(0);
                        modal_attributes.set(ModalAttributes { open: false, ..Default::default() });
                        return;
                    }
                    if message_data.as_string().unwrap() == "null" {
                        queue_data.set(HashMap::new());
                        return;
                    };
                    let parsed_data = serde_json::from_str::<HashMap<String, Value>>(
                        message_data.as_string().unwrap().as_str(),
                    ).unwrap();
                    // Fix so db_data does not return before db_conflict
                    if parsed_data.contains_key("db_data") {
                        let mut new_data = (*db_data).clone();
                        new_data.extend(
                            serde_json::from_value::<HashMap<String, Value>>(
                                parsed_data.get("db_data").unwrap().clone(),
                            )
                            .unwrap(),
                        );
                        db_data.set(new_data);
                        if parsed_data.contains_key("db_response") && parsed_data.get("db_response").unwrap().as_bool().unwrap() == true {
                            db_response_callback.emit(parsed_data.get("db_data").unwrap().clone());
                            db_response_callback.set(Default::default());
                        }
                        return;
                    }
                    if parsed_data.contains_key("db_response") {
                        db_response_callback.emit(parsed_data.get("db_response").unwrap().clone());
                        db_response_callback.set(Default::default());
                        return;
                    }
                    let mut new_data = (*queue_data).clone();
                    new_data.extend(parsed_data.into_iter());
                    queue_data.set(new_data);
                }) as Box<dyn FnMut(MessageEvent)>)
            };
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();
        }
    });
    // First Render
    use_effect_with((), {
        let mounted = mounted.clone();
        let active_services = default_props.active_services.clone();
        move |_| {
            spawn_local(async move {
                active_services.set(get_service_token_cookies().await);
                *mounted.borrow_mut() = true;
            });

        }
    });
    // Change playlists for playlist add page based on selected service
    use_effect_with(default_props.selected_service.clone(), {
        let mounted = mounted.clone();
        let active_services = default_props.active_services.clone();
        let user_service_data = default_props.user_service_data.clone();
        let playlists_index = default_props.service_playlists_index.clone();
        move |selected_service| {
            if *mounted.borrow_mut() {
                info!("service change");
                playlists_index.set(1);
                user_service_playlists_request(
                    active_services.clone(),
                    **selected_service,
                    user_service_data.clone(),
                    *playlists_index,
                );
            }
        }
    });
    html! {
        <div class="relative flex h-full w-full">
            <ContextProvider<DefaultProps> context={default_props.clone()}>
                <Modal />
                <NavBar guild_id={props.guild_id.clone()} />
                <Switch<Route> render={Callback::from(switch)} />
            </ContextProvider<DefaultProps>>
        </div>
    }
}
