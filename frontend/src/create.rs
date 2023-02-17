use common::DBPartProps;
use gloo_net::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::{Component, html, classes, Callback};
use web_sys::{Event, InputEvent, HtmlInputElement};

pub struct CreatePart {
    part: DBPartProps,
}

pub enum CreatePartMessage {
    UpdateName(String),
}

impl Component for CreatePart {
    type Message = CreatePartMessage;
    type Properties = ();

    fn create(_: &yew::Context<Self>) -> Self {
        Self {
            part: DBPartProps { 
                name: "".into(),
                ..Default::default()
            }
        }
    }

    fn update(&mut self, _: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CreatePartMessage::UpdateName(name) => self.part.name = name,
        }

        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let onclick = {
            let name = self.part.name.clone();

            Callback::from(move |_| {
            let json = DBPartProps {
                name: name.clone(),
                ..Default::default()
            };

            spawn_local(async move {
                let json = json.to_owned();
                Request::post("http://127.0.0.1:8088/api/part/create")
                    .json(&json)
                    .unwrap()
                    .send()
                    .await
                    .unwrap();
            });
        })};

        let callback = ctx.link().callback(move |name: String| {
            CreatePartMessage::UpdateName(name.clone())
        });

        let oninput = Callback::from(move |event: InputEvent| {
            let event: Event = event.dyn_into().unwrap();
            let event_target = event.target().unwrap();
            let html_element: HtmlInputElement = event_target.dyn_into().unwrap();
            callback.emit(html_element.value())
        });

        html! {
            <div class={classes!("create_part")}>
                <input type="text" {oninput} value={self.part.name.clone()} />
                <div onclick={onclick}>{"Submit"}</div>
            </div>
        }
    }
}
