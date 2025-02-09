use yew::{function_component, html, Html, use_context, use_effect_with, use_state, use_mut_ref, use_force_update, Callback};
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use gloo::net::http::Request;
use chrono::{DateTime, Utc};
use gloo::console::info;
use crate::lib::util::format_url;
use crate::guild::DefaultProps;

#[function_component(SettingsStatus)]
pub fn status() -> Html {
    let default_props = use_context::<DefaultProps>().expect("ctx");
    let backend_status = use_state(|| 0);
    let ping_latency = use_state(|| 0);
    let websocket_latency = use_state(|| -1);
    let websocket_test = {
        let ws = default_props.ws.clone();
        let websocket_latency = websocket_latency.clone();
        Callback::from(move |_| {
            let websocket_latency = websocket_latency.clone();
            let time = Utc::now();
            ws.send_with_str(r#"{"action":"ping"}"#).unwrap();
            default_props.db_response_callback.set(Callback::from(move |_| websocket_latency.set(Utc::now().signed_duration_since(time).num_milliseconds())))
        })
    };
    use_effect_with((), {
        let backend_status = backend_status.clone();
        let ping_latency = ping_latency.clone();
        move |_| {
            spawn_local(async move {
                let time = Utc::now();
                let response = Request::get(format_url("/api/ping").as_str()).send().await.expect("response");
                backend_status.set(response.status());
                ping_latency.set(
                    Utc::now().signed_duration_since(time).num_milliseconds()
                )
            });
        }
    });
    html! {
        <div class="h-full flex flex-row flex-wrap text-[1.4vw]">
            <div class="flex flex-col h-full w-1/2">
                <div class="w-full text-[2vw] font-semibold">
                    {"Webserver Status"}
                </div>
                <div class="flex flex-row justify-between items-center p-[0.5vw]">
                    <span class="w-full">{"Ping Response"}</span>
                    <div class="flex flex-row">
                        if *backend_status == 200 {
                            <span class="text-green-500">{"Success"}</span>
                        } else if *backend_status != 0 {
                            <span class="text-red-500">{"Error"}</span>
                        }
                    </div>
                </div>
                <div class="flex flex-row justify-between items-center p-[0.5vw]">
                    <span class="w-full">{"Ping Latency"}</span>
                    <div class="flex flex-row">
                        <span class="text-nowrap">{*ping_latency}{" ms"}</span>
                    </div>
                </div>
                <div class="w-full text-[2vw] font-semibold">
                    {"Bot Status"}
                </div>
                <div class="flex flex-row justify-between items-center p-[0.5vw]">
                    <span class="w-full">{"Websocket"}</span>
                    <div class="flex flex-row">
                        if default_props.ws.ready_state() == 1 {
                            <span class="text-green-500">{"Connected"}</span>
                        } else if default_props.ws.ready_state() == 0 {
                            <span class="text-yellow-500">{"Loading"}</span>
                        } else {
                            <span class="text-red-500">{"Disconnected"}</span>
                        }
                    </div>
                </div>
                <div class="flex flex-row justify-between items-center p-[0.5vw]">
                    <span class="w-full">{"Websocket Latency"}</span>
                    <div class="flex flex-row">
                        if *websocket_latency == -1 {
                            <button
                                class="bg-neutral-800 hover:bg-sky-950 hover:text-sky-500 rounded-md text-[1vw] text-nowrap p-[0.5vw] font-semibold duration-150"
                                onclick={websocket_test}
                            >{"Test Latency"}</button>
                        } else {
                            <span class="text-nowrap">{*websocket_latency}{" ms"}</span>
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}
