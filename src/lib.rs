extern crate stdweb;
use failure::Error;
use serde_derive::{Deserialize, Serialize};
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew::format::{Json};
use yew::services::Task;
use yew::services::websocket::{WebSocketService, WebSocketTask, WebSocketStatus};

pub enum Msg {
    WsAction(WsAction),
    WsReady(Result<WsResponse, Error>),
    WsOpened,
}

impl From<WsAction> for Msg {
    fn from(action: WsAction) -> Self {
        Msg::WsAction(action)
    }
}
pub enum WsAction {
    Disconnect,
    Lost,
}

pub struct Model {
    data: Option<String>,
    ws: Option<WebSocketTask>,
}
/// This type is used as a request which sent to websocket connection.
#[derive(Serialize, Debug)]
struct WsRequest {
    #[serde(rename = "type")] 
    req_type: String,
    product_ids: Vec<String>,
    channels: Vec<String>
}

/// This type is an expected response from a websocket connection.
#[derive(Deserialize, Debug)]
pub struct WsResponse {
    #[serde(rename = "type")] 
    req_type: String,
    best_bid: String,
    best_ask: String
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {

        let callback = link.send_back(|Json(data)| Msg::WsReady(data));
        let notification = link.send_back(|status| {
            match status {
                WebSocketStatus::Opened => Msg::WsOpened,
                WebSocketStatus::Closed | WebSocketStatus::Error => WsAction::Lost.into(),
            }
        });
        
        let mut ws_service = WebSocketService::new();
        let task = ws_service.connect("wss://ws-feed.gdax.com", callback, notification);
        
        Model {
            data: None,
            ws: Some(task),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::WsAction(action) => {
                match action {
                    WsAction::Disconnect => {
                        self.ws.take().unwrap().cancel();
                    }
                    WsAction::Lost => {
                        self.ws = None;
                    }
                }
            }
            Msg::WsReady(response) => {
                println!("Received: {:?}", response);
                self.data = response.map(|data| data.best_bid).ok();
            }
            Msg::WsOpened => {
                let request = WsRequest {
                    req_type: "subscribe".to_string(),
                    product_ids: vec!("BTC-USD".to_string()),
                    channels: vec!("ticker".to_string()),
                };
                self.ws.as_mut().unwrap().send(Json(&request));
                return false;
            }
        }
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <nav class="menu",>
                    { self.view_data() }
                    <button disabled=self.ws.is_none(),
                            onclick=|_| WsAction::Disconnect.into(),>{ "Close WebSocket connection" }</button>
                </nav>
            </div>
        }
    }

}


impl Model {
    fn view_data(&self) -> Html<Model> {
        if let Some(value) = &self.data {
            html! {
                <h2>{ value }</h2>
            }
        } else {
            html! {
                <h2>{ "Data hasn't fetched yet." }</h2>
            }
        }
    }
}