use std::rc::Rc;

use common::DBPart;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::{AppContext, AppRoute};

pub struct Parts {
    parts: Vec<Part>,
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

pub enum PartsMessage {
    ContextChanged(Rc<AppContext>),
    AddParts(Vec<Part>),
    SetSelected(Part, bool),
}

impl Component for Parts {
    type Message = PartsMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // if _listener is dropped no context update is provided
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(PartsMessage::ContextChanged))
            .unwrap();

        let callback = ctx.link().callback(move |parts| { PartsMessage::AddParts(parts) });
        spawn_local(get_parts_from_db(context.clone(), callback));

        Self { 
            parts: Vec::new(),
            context,
            _listener,
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PartsMessage::AddParts(mut parts) => {
                let selected_parts = &self.context.selected_parts;
                for part in parts.iter_mut() {
                    if selected_parts.contains(&part.id) {
                        part.selected = true;
                    }
                }

                self.parts.append(&mut parts);
            }
            PartsMessage::ContextChanged(context) => self.context = context,
            PartsMessage::SetSelected(part, selected) => {
                let part = self.parts.iter_mut().find(|x| **x == part);
                if let Some(part) = part {
                    part.selected = selected;
                    self.context.selected_parts_callback.emit((part.id.clone(), selected));
                }
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = ctx.link().callback(move |part: Part| {
            PartsMessage::SetSelected(part.clone(), !part.selected.clone())
        });

        let parts: Html = self.parts.iter().map(|part| html! {
            part.to_html(callback.clone())
        }).collect();

        html! {
            <div class={classes!("parts")}>
                {parts}
            </div>
        }
    }
}

async fn get_parts_from_db(context: Rc<AppContext>, callback: Callback<Vec<Part>>) {
    let parts = context.get_parts(2).await;
    callback.emit(parts.iter().map(|x| Part::from(x.clone())).collect());
}

#[derive(Clone)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub selected: bool,
}

impl PartialEq for Part {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Part {
    pub fn new<T>(id: String, name: T) -> Self 
    where T: Into<String>
    {
        Self { 
            id, 
            name: name.into(), 
            selected: false 
        }
    }

    fn to_html(&self, callback: Callback<Part>) -> Html {
        let on_click = {
            let callback = callback.clone();
            let part = self.clone();
            Callback::from(move |_| {
                callback.emit(part.clone())
            })
        };

        html! {
            <div class={classes!("part")}>
                <input type="checkbox" onclick={on_click} checked={self.selected} />
                <Link<AppRoute> to={AppRoute::Part { id: self.id.clone() }}>
                    <h3 class={classes!("part_name")}>{&self.name}</h3>
                </Link<AppRoute>>
            </div>
        }
    }
}

impl From<DBPart> for Part {
    fn from(value: DBPart) -> Self {
        Self {
            id: value.id,
            name: value.name,
            selected: false,
        }
    }
}
