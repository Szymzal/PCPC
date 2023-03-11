use std::{rc::Rc, collections::HashMap};

use common::PartsCategory;
use yew::prelude::*;

use crate::{app::AppContext, parts::Part};

pub struct Filter {
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

pub enum FilterMessage {
    ContextChanged(Rc<AppContext>),
    FilterVisibilityChanged((String, bool)),
    CategorySelectedChanged(String),
}

impl Component for Filter {
    type Message = FilterMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(FilterMessage::ContextChanged))
            .unwrap();

        let selected_category = PartsCategory::from_string(&context.selected_category);
        let ordering = ordering(selected_category);
        if context.properties_order.is_empty() {
            context.properties_order_callback.emit(ordering.clone());
        }

        Self {
            context,
            _listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FilterMessage::ContextChanged(context) => self.context = context,
            FilterMessage::FilterVisibilityChanged((name, selected)) => {
                let context = &self.context;
                let mut properties_order = context.properties_order.clone();
                properties_order.insert(name, selected);
                context.properties_order_callback.emit(properties_order);
            },
            FilterMessage::CategorySelectedChanged(category_string) => {
                if self.context.selected_category != category_string {
                    let category = PartsCategory::from_string(&category_string);
                    let mut new_order = self.context.properties_order.clone();
                    let ordering = ordering(category);
                    for order in ordering {
                        if !new_order.contains_key(&order.0) {
                            new_order.insert(order.0, order.1);
                        }
                    }
                    self.context.properties_order_callback.emit(new_order);
                    self.context.selected_category_callback.emit(category_string);
                }
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let categories = categories();
        let mut categories_html: Vec<Html> = Vec::new();
        for category in &categories {
            let callback = ctx.link().callback(move |(category, _selected)| FilterMessage::CategorySelectedChanged(category));
            let selected = if self.context.selected_category == category.clone() { true } else { false };

            categories_html.push(html! {
                <Property name={category.clone()} selected={selected} callback={callback} />
            });
        }

        let mut ordering_properties: Vec<Html> = Vec::new();
        let selected_category = PartsCategory::from_string(&self.context.selected_category);
        let template_part = Part {
            category_properties: selected_category,
            ..Default::default()
        };
        let template_properties = template_part.get_properties_as_map().unwrap_or(HashMap::new());
        let mut properties_order = self.context.properties_order.clone();

        properties_order.retain(|key, _| template_properties.contains_key(key));

        for (name, selected) in properties_order {
            let callback = ctx.link().callback(move |(name, selected)| FilterMessage::FilterVisibilityChanged((name, selected)));
            ordering_properties.push(html! {
                <Property name={name.to_owned()} selected={selected} callback={callback} />
            });
        }

        html! {
            <div class={classes!("side-panel")}>
                <div class={classes!("filter")}>
                    <h2>{"Category"}</h2>
                    {categories_html}
                    <h2>{"Properties"}</h2>
                    {ordering_properties}
                </div>
                <div 
                    class={classes!("resizer-right")}>
                </div>
            </div>
        }
    }
}

fn categories() -> Vec<String> {
    return PartsCategory::get_all_variats();
}

fn ordering(category: PartsCategory) -> HashMap<String, bool> {
    let part_template = Part {
        category_properties: category,
        ..Default::default()
    };
    let part_field_names = part_template.get_properties_as_map();
    if let Ok(part_field_names) = part_field_names {
        let mut map: HashMap<String, bool> = HashMap::new();
        for key in part_field_names.keys() {
            map.insert(key.to_string(), true);
        }
        return map;
    }

    HashMap::new()
}

pub struct Property;

#[derive(Properties, Clone, PartialEq)]
pub struct PropertyProps {
    pub name: String,
    pub selected: bool,
    pub callback: Callback<(String, bool)>,
}

pub enum PropertyMessage {
    ChangeSelected(bool),
}

impl Component for Property {
    type Message = PropertyMessage;
    type Properties = PropertyProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PropertyMessage::ChangeSelected(selected) => { 
                let props = ctx.props();
                props.callback.emit((props.name.clone(), selected));
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_click = {
            let props = ctx.props().clone();
            ctx.link().callback(move |_| PropertyMessage::ChangeSelected(!props.selected))
        };

        html! {
            <div class={classes!("ordering-property")}>
                <input type="checkbox" onchange={on_click} checked={ctx.props().selected} />
                <h3>{ctx.props().name.clone()}</h3>
            </div>
        }
    }
}
