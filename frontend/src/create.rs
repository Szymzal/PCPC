use std::{collections::HashMap, rc::Rc};

use anyhow::bail;
use common::{DBPartProps, traits::PartProperties, PartsCategory};
use gloo_net::http::Request;
use serde_json::{Value, Map};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::{Component, html, classes, Callback, Properties, Html, ContextHandle};
use web_sys::{Event, InputEvent, HtmlInputElement, HtmlSelectElement, RequestCredentials};
use yew_router::prelude::Redirect;

use crate::app::{AppContext, AppRoute};

pub struct CreatePart {
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
    part: HashMap<String, String>,
    selected_category: PartsCategory,
    auth_required: bool,
}

pub enum CreatePartMessage {
    ContextChanged(Rc<AppContext>),
    Update(String, String),
    SetSelectedCategory(PartsCategory),
    AuthRequired,
}

impl Component for CreatePart {
    type Message = CreatePartMessage;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(CreatePartMessage::ContextChanged))
            .unwrap();

        let default_part = DBPartProps::default();
        let mut map = part_props_get_properties_as_map(default_part).unwrap();
        map.remove("Category");

        Self {
            context,
            _listener,
            part: map,
            selected_category: PartsCategory::default(),
            auth_required: false,
        }
    }

    fn update(&mut self, _: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CreatePartMessage::Update(key, value) => { self.part.insert(key, value); },
            CreatePartMessage::ContextChanged(context) => self.context = context,
            CreatePartMessage::SetSelectedCategory(category) => { 
                let default_part = DBPartProps {
                    category: category.clone(),
                    ..Default::default()
                };
                let mut default_map = part_props_get_properties_as_map(default_part).unwrap();
                default_map.remove("Category");

                let part = self.part.clone();
                let keys = part.keys().clone();
                for key in keys {
                    if !default_map.contains_key(key) {
                        self.part.remove(key);
                    }
                }

                for (key, value) in default_map {
                    if !self.part.contains_key(&key) {
                        self.part.insert(key, value);
                    }
                }

                self.selected_category = category 
            },
            CreatePartMessage::AuthRequired => self.auth_required = true,
        }

        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let onclick = {
            let context = self.context.clone();
            let auth_callback = context.auth_callback.clone();
            let auth_required_callback = ctx.link().callback(move |()| CreatePartMessage::AuthRequired);
            let map = self.part.clone();
            let selected_category = self.selected_category.clone();

            Callback::from(move |_| {
                let json = get_json(&map, &selected_category).unwrap();
                let auth_callback = auth_callback.clone();
                let auth_required_callback = auth_required_callback.clone();
                spawn_local(async move {
                    let json = json.to_owned();
                    let request = Request::post("http://127.0.0.1:8088/api/part/create")
                        .json(&json)
                        .unwrap()
                        .send()
                        .await
                        .unwrap();
                    if request.status() == 401 {
                        let value = serde_json::to_value(json).unwrap();
                        auth_callback.emit(Some((AppRoute::Create, value)));
                        auth_required_callback.emit(());
                    }
                });
            }
        )};

        let callback = ctx.link().callback(move |(name, value): (String, String)| {
            CreatePartMessage::Update(name, value)
        });

        let mut inputs: Vec<Html> = Vec::new();
        let mut unused = self.part.clone();
        for key in self.context.properties_order.keys() {
            let value = self.part.get(key);
            unused.remove(key);
            if let Some(value) = value {
                inputs.push(html! {
                    <PropertyInput callback={callback.clone()} name={key.clone()} value={value.clone()} />
                });
            }
        }

        for (key, value) in unused {
            inputs.push(html! {
                <PropertyInput callback={callback.clone()} name={key.clone()} value={value.clone()} />
            });
        }

        let mut categories_html: Vec<Html> = Vec::new();
        for category in PartsCategory::get_all_variats() {
            if self.selected_category == PartsCategory::from_string(&category) {
                categories_html.push(html! {
                    <option selected={true} value={category.clone()}>{category.clone()}</option>
                });

                continue;
            }
        
            categories_html.push(html! {
                <option value={category.clone()}>{category.clone()}</option>
            });
        }

        let select_callback = ctx.link().callback(move |category| CreatePartMessage::SetSelectedCategory(category));
        let select_on_change = Callback::from(move |input_event: InputEvent| {
            let event: Event = input_event.dyn_into().unwrap();
            let event_target = event.target().unwrap();
            let html_element: HtmlSelectElement = event_target.dyn_into().unwrap();
            let category = PartsCategory::from_string(&html_element.value());
            select_callback.emit(category);
        });

        html! {
            <div class={classes!("create-part")}>
                <div class={classes!("create-part-selection-box")}>
                    <label for={"part-category"}>{"Part category:"}</label>
                    <select name={"part-category"} class={classes!("part-category-selection")} oninput={select_on_change}>
                        {categories_html}
                    </select>
                </div>

                <div class={classes!("properties")}>
                    {inputs}
                </div>
                <div class={classes!("create-part-button")} onclick={onclick}>
                    <h2>{"Submit"}</h2>
                </div>
                if self.auth_required {
                    <Redirect<AppRoute> to={AppRoute::Auth} />
                }
            </div>
        }
    }
}

pub struct PropertyInput;

#[derive(Properties, Clone, PartialEq)]
pub struct PropertyInputProps {
    callback: Callback<(String, String)>,
    name: String,
    value: String,
}

impl Component for PropertyInput {
    type Message = ();
    type Properties = PropertyInputProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();
        let callback = props.callback.clone();
        let oninput = { 
            let name = props.name.clone();
            Callback::from(move |event: InputEvent| {
                let event: Event = event.dyn_into().unwrap();
                let event_target = event.target().unwrap();
                let html_element: HtmlInputElement = event_target.dyn_into().unwrap();
                callback.emit((name.clone(), html_element.value()));
            }
        )};

        html! {
            <div class={classes!("property-input")}>
                <p>{&props.name}</p>
                <input type="text" {oninput} value={props.value.clone()}/>
            </div>
        }
    }
}

fn get_json(part: &HashMap<String, String>, selected_category: &PartsCategory) -> anyhow::Result<DBPartProps> {
    let mut result = "{".to_string();
    let default_part = DBPartProps::default();
    let part_value = serde_json::to_value(default_part)?;
    let selected_category = selected_category.clone();
    let default_category = selected_category.clone();
    let category_value = serde_json::to_value(default_category)?;

    let mut stored_map = part.clone();
    for (old_key, value) in stored_map.clone() {
        let mut key = old_key.clone();
        let mut chars: Vec<char> = key.chars().collect();
        let lowercase_char = chars[0].to_lowercase().nth(0);
        if let Some(lowercase_char) = lowercase_char {
            chars[0] = lowercase_char;
            key = chars.into_iter().collect();
        }
        key = key.replace(" ", "_");
        
        let value = value.trim();

        stored_map.remove(&old_key);
        stored_map.insert(key, value.to_string());
    }


    match part_value {
        serde_json::Value::Object(mut part_map) => {
            let mut category_map: Map<String, Value>;
            match category_value {
                serde_json::Value::String(_) => {
                    category_map = Map::default();
                }
                serde_json::Value::Object(object) => {
                    category_map = object;
                },
                _ => bail!("No way"),
            }

            let first_key: Vec<&String> = category_map.keys().collect();
            if !first_key.is_empty() {
                let first = first_key.first();
                if let Some(first) = first {
                    let value = category_map.get(*first);
                    if let Some(value) = value {
                        match value {
                            serde_json::Value::Object(object) => category_map = object.clone(),
                            _ => bail!("Unexpected error on category"),
                        }
                    }
                }
            }

            part_map.remove("category");

            let mut index = 0;
            for map in [part_map, category_map] {
                let mut map_index = 0;
                let map_len = map.len();

                for (key, value) in map {
                    let stored_value = stored_map.get(&key);

                    if let Some(stored_value) = stored_value {
                        result.push_str(&format!("\"{}\":", key));
                        
                        match value {
                            serde_json::Value::Bool(_) => {
                                let bool_value: Result<bool, _> = stored_value.parse();
                                
                                if let Ok(bool_value) = bool_value {
                                    result.push_str(&bool_value.to_string());
                                } else {
                                    bail!("Property {}, has wrong value! Expected bool", &key);
                                }
                            },
                            serde_json::Value::Number(value) => {
                                if value.is_u64() {
                                    let u64_value: Result<u64, _> = stored_value.parse();

                                    if let Ok(u64_value) = u64_value {
                                        result.push_str(&u64_value.to_string());
                                    } else {
                                        bail!("Property {}, has wrong value! Expected u64", &key);
                                    }
                                } else if value.is_i64() {
                                    let i64_value: Result<i64, _> = stored_value.parse();
                                    
                                    if let Ok(i64_value) = i64_value {
                                        result.push_str(&i64_value.to_string());
                                    } else {
                                        bail!("Property {}, has wrong value! Expected i64", &key);
                                    }
                                } else {
                                    let f64_value: Result<f64, _> = stored_value.parse();
                                    
                                    if let Ok(f64_value) = f64_value {
                                        result.push_str(&f64_value.to_string());
                                    } else {
                                        bail!("Property {}, has wrong value! Expected f64", &key);
                                    }
                                }
                            },
                            serde_json::Value::String(_) => {
                                result.push_str(&format!("\"{}\"", stored_value.as_str()));
                            },
                            _ => bail!("What"),
                        }

                        if map_index != map_len - 1 {
                            result.push_str(",");
                        }
                    }

                    map_index += 1;
                }

                if index == 0 {
                    let string_category = selected_category.to_string();
                    if selected_category == PartsCategory::Basic {
                        result.push_str(",\"category\":\"Basic\"");
                    } else {
                        result.push_str(&format!(",\"category\":{{\"{}\":{{", string_category));
                    }
                }

                if index == 1 {
                    if selected_category != PartsCategory::Basic {
                        result.push_str("}}");
                    }
                }

                index += 1;
            }
        },
        _ => bail!("HOW"),
    }

    result.push_str("}");

    let part_props_struct: DBPartProps = serde_json::from_str(&result)?;

    Ok(part_props_struct)
}

fn part_props_get_properties_as_map(props: DBPartProps) -> anyhow::Result<HashMap<String, String>> {
    let props_stripped = DBPartProps {
        category: PartsCategory::default(),
        ..props
    };

    let mut base_map = props_stripped.to_string_vec()?;
    let category_properties_map = props.category.to_string_vec()?;
    
    base_map.extend(category_properties_map);

    Ok(base_map)
}
