use std::rc::Rc;

use common::{DBPart, PartsCategory};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::{AppContext, AppRoute, get_parts_with_callback};

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
        spawn_local(get_parts_with_callback(context.clone(), 20, callback));

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

#[derive(Clone)]
pub struct Part {
    pub id: String,
    pub selected: bool,
    pub favorited: bool,
    pub name: String,
    pub image_url: String,
    pub model: String,
    pub manufactuer: String,
    pub release_date: u64,
    pub rating: f32,
    pub category_properties: PartsCategory,
}

impl PartialEq for Part {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Part {
    pub fn new<T>(
        id: String, 
        name: T,
        image_url: T,
        model: T,
        manufactuer: T,
        release_date: u64,
        rating: f32,
        category: PartsCategory,
    ) -> Self 
    where T: Into<String>
    {
        Self { 
            id, 
            selected: false,
            favorited: false,
            name: name.into(), 
            image_url: image_url.into(),
            model: model.into(),
            manufactuer: manufactuer.into(),
            release_date,
            rating,
            category_properties: category,
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

impl Default for Part {
    fn default() -> Self {
        Self { 
            id: "".into(), 
            selected: false, 
            favorited: false, 
            name: "".into(), 
            image_url: "".into(), 
            model: "".into(), 
            manufactuer: "".into(), 
            release_date: 0, 
            rating: 0.0,
            category_properties: PartsCategory::Basic,
        }
    }
}

impl From<DBPart> for Part {
    fn from(value: DBPart) -> Self {
        Self::new(
            value.id, 
            value.name, 
            value.image_url, 
            value.model, 
            value.manufactuer, 
            value.release_date, 
            value.rating,
            PartsCategory::Basic,
        )
    }
}
