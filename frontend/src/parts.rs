use std::{rc::Rc, collections::HashMap};

use common::{DBPart, PartsCategory, traits::PartProperties};
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{app::{AppContext, get_parts_with_callback}, filter::Filter, icons::SearchBar, rating::Rating};

pub struct Parts {
    parts: Vec<Part>,
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

pub enum PartsMessage {
    ContextChanged(Rc<AppContext>),
    AddParts(Vec<Part>),
    SetSelected(String, bool),
    SetFavorite(String, bool),
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
                let favorited_parts = &self.context.favorites;
                for part in parts.iter_mut() {
                    if selected_parts.contains(&part.id) {
                        part.selected = true;
                    }

                    if favorited_parts.contains(&part.id) {
                        part.favorited = true;
                    }
                }

                self.parts.append(&mut parts);
            }
            PartsMessage::ContextChanged(context) => self.context = context,
            PartsMessage::SetSelected(part_id, selected) => {
                let part = self.parts.iter_mut().find(|x| x.id == part_id);
                if let Some(part) = part {
                    part.selected = selected;
                    self.context.selected_parts_callback.emit((part.id.clone(), selected));
                }
            },
            PartsMessage::SetFavorite(part_id, favorited) => {
                let part = self.parts.iter_mut().find(|x| x.id == part_id);
                if let Some(part) = part {
                    part.favorited = favorited;
                    self.context.favorites_callback.emit((part.id.clone(), favorited));
                }
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback_selected = ctx.link().callback(move |(part_id, selected): (String, bool)| PartsMessage::SetSelected(part_id.clone(), !selected));
        let callback_favorite = ctx.link().callback(move |(part_id, selected): (String, bool)| PartsMessage::SetFavorite(part_id.clone(), !selected));

        let mut ordering_properties = self.context.properties_order.clone();
        let parts: Html = self.parts.iter().map(|part| {
            if self.context.selected_category == part.category_properties.to_string() &&
                part.name.to_lowercase().contains(&self.context.search_term.to_lowercase()) {
                ordering_properties.retain(|_, selected| *selected);
                let ordering_properties: Vec<&String> = ordering_properties.keys().collect();
                return part.to_html(Some(&ordering_properties), callback_selected.clone(), callback_favorite.clone());
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
    pub release_date: String,
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
        release_date: String,
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

    pub fn get_properties_as_map(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut base_map = self.to_string_vec()?;
        let category_properties_map = self.category_properties.to_string_vec()?;
        
        base_map.extend(category_properties_map);

        Ok(base_map)
    }

    pub fn to_html(&self, order: Option<&Vec<&String>>, callback_selected: Callback<(String, bool)>, callback_favorite: Callback<(String, bool)>) -> Html {
        let on_click_selected = {
            let callback = callback_selected.clone();
            let part_id = self.id.clone();
            let part_selected = self.selected;
            Callback::from(move |_| {
                callback.emit((part_id.clone(), part_selected))
            })
        };

        let on_click_favorite = {
            let callback = callback_favorite.clone();
            let part_id = self.id.clone();
            let part_favorited = self.favorited;
            Callback::from(move |_| {
                callback.emit((part_id.clone(), part_favorited))
            })
        };

        let map = self.get_properties_as_map();
        let mut properties: Vec<Html> = Vec::new();
        if let (Ok(map), Some(order)) = (map, order) {
            for key in order {
                let value = map.get(*key);
                if let Some(value) = value {
                    let value = format_property(value.to_owned());
                    properties.push(html! {
                        <div class={classes!("part_specification")}>
                            <h4>{format!("{}:", key.to_owned())}</h4>
                            <h5>{value}</h5>
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
                        <h3 class={classes!("part_name")}>{&self.name}</h3>
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
                        <div class={classes!("part_action")} onclick={on_click_favorite}>
                            <img 
                                src= { if !self.favorited { "https://cdn-icons-png.flaticon.com/512/1077/1077035.png" } else { "https://cdn-icons-png.flaticon.com/512/1077/1077086.png" } }
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
            release_date: "".to_string(), 
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

pub fn format_property(property: String) -> String {
    match property.as_str() {
        "true" => "Yes".to_string(),
        "false" => "No".to_string(),
        _ => property,
    }
}
