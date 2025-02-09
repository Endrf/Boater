use crate::lib::api::api_refresh_service_token;
use chrono::{NaiveDate, TimeDelta, Utc};
use gloo::console::info;
use std::collections::HashMap;
use std::ops::Add;
use std::time::Duration;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlDocument};
use yew::{html, Html};

pub fn format_url(url: &str) -> String {
    let is_discord_proxy = window().expect("window").location().hostname().expect("url").contains("1140722929374613556");
    if url.is_empty() {
        return String::new();
    } else if !is_discord_proxy {
        return url.to_string();
    } else if url.contains("https://") || url.contains("wss://") {
        String::from("/.proxy") + &*url[7..].replacen('.', "/", 2)
    } else {
        "/.proxy".to_owned() + url
    }
}

pub fn format_time(duration_ms: u64) -> String {
    let duration = Duration::from_millis(duration_ms);
    let mut seconds = duration.as_secs();
    let mut minutes: u64 = 0;
    if seconds >= 60 {
        minutes = seconds / 60;
        seconds -= minutes * 60;
    }

    format!("{:0>2}:{:0>2}", minutes, seconds)
}

pub fn format_release(date: &str) -> String {
    let precision = date.split('-').collect::<Vec<&str>>().len();
    let date_precise = if precision == 3 {
        date.to_owned()
    } else if precision == 2 {
        date.to_owned() + "-1"
    } else {
        date.to_owned() + "-1-1"
    };
    let track_date = match NaiveDate::parse_from_str(date_precise.as_str(), "%Y-%m-%d").ok() {
        None => return String::from("Error Retrieving Date"),
        Some(track_date) => track_date,
    }; // Panic Error
    let duration = Utc::now()
        .naive_utc()
        .date()
        .signed_duration_since(track_date);
    let mut days = duration.num_days();
    let mut months = 0;
    let mut years = 0;
    if days >= 30 {
        months = days / 30;
        days -= months * 30;
    }
    if months >= 12 {
        years = months / 12;
        months -= years * 12;
    }

    if years == 1 {
        "1 year ago".to_string()
    } else if years > 1 {
        format!("{} years ago", years)
    } else if months == 1 {
        "1 month ago".to_string()
    } else if months > 1 {
        format!("{} months ago", months)
    } else if days == 1 {
        "1 day ago".to_string()
    } else if days > 1 {
        format!("{} days ago", days)
    } else {
        format!("{} days ago", days)
    }
}

pub fn open_link(url: String) {
    if crate::getGuildId() == *"" {
        window().expect("window").open_with_url_and_target_and_features(&url, "_blank", "popup,width=400,height=540")
            .unwrap().unwrap().focus().unwrap();
    } else {
        crate::openExternalLink(url);
    }
}

pub fn get_cookie(key: &str) -> Option<String> {
    let document = window().expect("window").document().unwrap().dyn_into::<HtmlDocument>().expect("document");
    if document.cookie().expect("cookies").as_str() == "" {
        return None;
    }
    document.cookie().expect("cookies").as_str().split("; ").find(|&value| value.split_once('=').unwrap().0 == key)
        .map(|x| x.split_once('=').unwrap().1.to_string())
}

pub fn set_service_token_cookie(service: &str, token: &str, expires: i64, refresh: &str) {
    let document = window().expect("window").document().unwrap().dyn_into::<HtmlDocument>().expect("document");
    let no_expire_time = Utc::now().add(TimeDelta::days(365)).format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let expire_time = Utc::now().add(TimeDelta::seconds(expires - 100)).format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    document.set_cookie(&format!("__Host-{service}_token={token}; SameSite=None; Path=/; expires={expire_time}; Secure; Partitioned;")).unwrap();
    document.set_cookie(&format!("__Host-{service}_refresh={refresh}; SameSite=None; Path=/; expires={no_expire_time}; Secure; Partitioned;")).unwrap();
}

pub fn remove_service_token_cookie(service: &str) {
    let document = window().expect("window").document().unwrap().dyn_into::<HtmlDocument>().expect("document");
    document.set_cookie(&format!("__Host-{service}_token=; SameSite=None; Path=/; Secure; Partitioned; Max-Age=-99999999;")).unwrap();
    document.set_cookie(&format!("__Host-{service}_refresh=; SameSite=None; Path=/; Secure; Partitioned; Max-Age=-99999999;")).unwrap();
}

pub async fn get_service_token_cookies() -> HashMap<&'static str, Option<String>> {
    let services = vec!["spotify", "youtube", /*"soundcloud", "deezer",*/ "tidal"];
    let mut tokens: HashMap<&'static str, Option<String>> = Default::default();
    for x in services {
        let token = get_cookie(format!("__Host-{x}_token").as_str());
        if token.is_some() {
            tokens.insert(x, token);
        } else if let Some(refresh) = get_cookie(format!("__Host-{x}_refresh").as_str()) {
            let data = api_refresh_service_token(x, refresh).await;
            set_service_token_cookie(x, data.get("access_token").unwrap().as_str().unwrap(),
                data.get("expires_in").unwrap().as_i64().unwrap(), data.get("refresh_token").unwrap().as_str().unwrap()
            );
            tokens.insert(x, Some(data.get("access_token").unwrap().as_str().unwrap().to_string()));
        } else {
            tokens.insert(x, None);
        }
    }
    info!("test");
    tokens
}

pub fn service_icon_html(service: String) -> Html {
    match service.as_str() {
        "spotify" | "apple" | "youtube" => html! {<i class={format!("fa-brands fa-{}", service)} />},
        "boater" => html! {<i class="fa-solid fa-sailboat" />},
        "tidal" => html! {
            <svg class="inline" version="1.1" xmlns="http://www.w3.org/2000/svg" /*xmlns:xlink="http://www.w3.org/1999/xlink" x="0px" y="0px"*/ width="1em" height="1em" viewBox="0 0 176 177" /*enable-background="new 0 0 176 177" xml:space="preserve"*/>
                <g>
                    <g>
                        <path fill="currentColor" d="M-0.489,59.215L28.787,88.42l29.276-29.205L28.787,30.011L-0.489,59.215z M58.224,59.215L87.5,88.42l29.276-29.205 L87.5,30.011L58.224,59.215z M146.213,30.011l-29.276,29.205l29.276,29.205l29.276-29.205L146.213,30.011z M58.224,117.784 L87.5,146.989l29.276-29.205L87.5,88.58L58.224,117.784z"/>
                    </g>
                </g>
            </svg>
        },
        _ => html! {<></>},
    }
}
