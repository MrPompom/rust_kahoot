use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::{services::websocket::WebsocketService};
extern crate iron;
use wasm_bindgen::JsCast;

pub enum Msg {
    SubmitMessage(i32),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MsgTypes {
    Join(WebSocketConnexion),
    Start,
    Reponse(WebSocketMessage),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    question_id: i64,
    data: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketConnexion {
    code: String,
}

pub struct Chat {
    chat_input: NodeRef,
    wss: WebsocketService,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut wss = WebsocketService::new();
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let html_document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
        let temp = html_document.url().unwrap();
        let message = MsgTypes::Join(WebSocketConnexion {
            code: temp,
        });

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            chat_input: NodeRef::default(),
            wss,
        }
        
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SubmitMessage(index) => {
                let input = Some(index);
                if let Some(input) = input {
                    let message = MsgTypes::Reponse(WebSocketMessage {
                        data: Some(input.to_string()),
                        question_id: 0,
                    });
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let started = false;

        html! {
            <div class="flex w-screen">
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-gray-300"><div class="text-xl p-3">{"💬 Chat!"}</div></div>
                    <div class="container mx-auto flex flex-col justify-center items-center">
                        <div class="flex">
                            <button onclick={ctx.link().callback(|_| Msg::SubmitMessage(0))} class="px-8 rounded-lg bg-violet-600 text-white font-bold p-4 uppercase border-violet-600 border-t border-b border-r">
                                {"réponse A"}
                            </button>
                            <button onclick={ctx.link().callback(|_| Msg::SubmitMessage(1))} class="px-8 rounded-lg bg-violet-600 text-white font-bold p-4 uppercase border-violet-600 border-t border-b border-r">
                                {"réponse b"}
                            </button>
                        </div>
                        <div class="flex">
                            <button onclick={ctx.link().callback(|_| Msg::SubmitMessage(2))} class="px-8 rounded-lg bg-violet-600 text-white font-bold p-4 uppercase border-violet-600 border-t border-b border-r">
                                {"réponse c"}
                            </button>
                            <button onclick={ctx.link().callback(|_| Msg::SubmitMessage(3))} class="px-8 rounded-lg bg-violet-600 text-white font-bold p-4 uppercase border-violet-600 border-t border-b border-r">
                                {"réponse d"}
                            </button>
                        </div>
                    </div>
                        <div>
                            <button class="px-8 rounded-r-lg bg-violet-600	  text-white font-bold p-4 uppercase border-violet-600 border-t border-b border-r" >{"Join!"}</button>
                        </div>
                </div>
            </div>
        }
    }
}