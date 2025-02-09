use crate::routes::guild::DefaultProps;
use web_sys::MouseEvent;
use yew::{function_component, html, use_context, Callback, Html, Properties};

#[derive(PartialEq, Debug, Clone)]
pub enum ModalType {
    Confirmation,
    Selection,
    Notification,
    Loading,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ModalColor {
    Neutral,
    Primary,
    Danger,
    Warning,
    Success,
}

fn modal_color(modal_type: &ModalColor) -> String {
    match modal_type {
        ModalColor::Neutral => String::from("bg-neutral-900/70"),
        ModalColor::Primary => String::from("bg-sky-800/70"),
        ModalColor::Danger => String::from("bg-red-800/70"),
        ModalColor::Warning => String::from("bg-yellow-600/70"),
        ModalColor::Success => String::from("bg-green-800/70"),
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct ModalAttributes {
    pub title: String,
    pub content: Html,
    pub modal_type: ModalType,
    pub color: ModalColor,
    pub confirm: Callback<MouseEvent>,
    pub open: bool,
}

impl Default for ModalAttributes {
    fn default() -> ModalAttributes {
        ModalAttributes {
            title: Default::default(),
            content: Default::default(),
            modal_type: ModalType::Confirmation,
            color: ModalColor::Neutral,
            confirm: Default::default(),
            open: true,
        }
    }
}

#[function_component(Modal)]
pub fn modal() -> Html {
    let default_props = use_context::<DefaultProps>().expect("ctx");
    let attributes = default_props.modal_attributes;
    html! {
        if attributes.open {
            <dialog class="absolute flex justify-center items-center h-full w-full bg-neutral-900/20 backdrop-blur-sm shadow z-40" open=true>
                <div class="flex flex-col h-fit w-[38vw] bg-neutral-800 rounded-lg">
                    <div class={format!("flex items-center justify-between h-[4vw] w-full shadow {} rounded-t-lg", modal_color(&attributes.color))}>
                        <span class="p-[1vw] text-[2vw] font-semibold">{&attributes.title}</span>
                        if attributes.modal_type == ModalType::Selection || attributes.modal_type == ModalType::Notification {
                            <button
                                class="p-[1vw] text-[1.6vw] hover:text-red-700 duration-150"
                                onclick={let attributes = attributes.clone(); Callback::from(move |_| attributes.set(ModalAttributes { open: false, ..Default::default() }))}><i class="fa-solid fa-x" /
                            ></button>
                        }
                    </div>
                    <div class="flex flex-col h-full p-[1vw] gap-[1vw] text-[1.6vw]">
                        <div class="h-full flex text-center justify-center items-center">{&attributes.content}</div>
                        if attributes.modal_type == ModalType::Confirmation || attributes.modal_type == ModalType::Notification {
                            <div class="h-fit font-semibold">
                                <button class="float-right bg-green-900 hover:bg-green-800 p-[0.6vw] rounded transition-colors duration-150" onclick={&attributes.confirm}>
                                    if attributes.modal_type == ModalType::Confirmation {
                                        <i class="fas fa-check mr-[0.6vw]" />
                                        {"Confirm"}
                                    } else { {"Ok"} }
                                </button>
                                if attributes.modal_type == ModalType::Confirmation {
                                    <button class="float-right mr-[1vw] bg-red-900 hover:bg-red-800 p-[0.6vw] rounded transition-colors duration-150" onclick={Callback::from(move |_| attributes.set(ModalAttributes { open: false, ..Default::default() }))}>
                                        <i class="fas fa-x mr-[0.6vw]" />
                                        {"Cancel"}
                                    </button>
                                }
                            </div>
                        }
                    </div>
                </div>
            </dialog>
        }
    }
}
