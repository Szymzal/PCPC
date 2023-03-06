use std::{rc::Rc, collections::HashMap};

use common::{DBPart, PartsCategory, traits::PartProperties};
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{app::{AppContext, AppRoute, get_parts_with_callback}, filter::Filter, icons::SearchBar, rating::Rating};

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

        let ordering_properties = &self.context.properties_order;
        let parts: Html = self.parts.iter().map(|part| {
            if self.context.selected_category == part.category_properties.to_string() {
                return part.to_html(Some(ordering_properties), callback.clone());
            }
            
            return html! {}
        }).collect();

        html! {
            <div class={classes!("parts-page")}>
                if self.context.filter_visibility {
                    <Filter />
                }
                <div class={classes!("parts-container")}>
                    <SearchBar />
                    <div class={classes!("parts")}>
                        {parts}
                    </div>
                </div>
            </div>
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Part {
    #[serde(skip_serializing)]
    pub id: String,
    #[serde(skip_serializing)]
    pub selected: bool,
    #[serde(skip_serializing)]
    pub favorited: bool,
    #[serde(skip_serializing)]
    pub name: String,
    #[serde(skip_serializing)]
    pub image_url: String,
    pub model: String,
    pub manufactuer: String,
    pub release_date: u64,
    #[serde(skip_serializing)]
    pub rating: f32,
    #[serde(skip_serializing)]
    pub category_properties: PartsCategory,
}

impl PartProperties for Part {}

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

    fn get_properties_as_map(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut base_map = self.to_string_vec()?;
        let category_properties_map = self.category_properties.to_string_vec()?;

        base_map.extend(category_properties_map);

        Ok(base_map)
    }

    fn to_html(&self, order: Option<&Vec<String>>, callback: Callback<Part>) -> Html {
        let on_click_selected = {
            let callback = callback.clone();
            let part = self.clone();
            Callback::from(move |_| {
                callback.emit(part.clone())
            })
        };

        let map = self.get_properties_as_map();
        let mut properties: Vec<Html> = Vec::new();
        if let (Ok(map), Some(order)) = (map, order) {
            for key in order {
                let value = map.get(key);
                if let Some(value) = value {
                    properties.push(html! {
                        <div class={classes!("part_specification")}>
                            <h4>{format!("{}:", key.to_owned())}</h4>
                            <h5>{value.to_owned()}</h5>
                        </div>
                    });
                }
            }
        }

        html! {
            <div class={classes!("part")}>
                <div class={classes!("part_img")}>
                    <img src={self.image_url.clone()} alt="PC Part Image" />
                </div>
                <div class={classes!("part_content")}>
                    <div class={classes!("part_header")}>
                        <Link<AppRoute> to={AppRoute::Part { id: self.id.clone() }}>
                            <h3 class={classes!("part_name")}>{&self.name}</h3>
                        </Link<AppRoute>>
                    </div>
                    <div class={classes!("part_info")}>
                        <Rating rating={self.rating} />
                        {properties}
                    </div>
                    <div class={classes!("part_footer")}>
                        <div class={classes!("part_action")} onclick={on_click_selected} >
                            <img 
                                src={ if !self.selected { "https://cdn-icons-png.flaticon.com/512/3524/3524388.png" } else { "https://cdn-icons-png.flaticon.com/512/56/56889.png" } }
                                alt="Select" 
                            />
                        </div>
                        <div class={classes!("part_action")}>
                            <img 
                                src="https://cdn-icons-png.flaticon.com/512/1077/1077035.png"
                                alt="Favorite"
                            />
                        </div>
                    </div>
                </div>
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
            value.rating.into(),
            value.category,
        )
    }
}
