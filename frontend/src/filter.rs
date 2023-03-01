use std::rc::Rc;

use serde_json::Value;
use yew::prelude::*;

use crate::{app::AppContext, parts::Part};

pub struct Filter {
    ordering: Vec<String>,
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

pub enum FilterMessage {
    ContextChanged(Rc<AppContext>),
    FilterVisibilityChanged((String, bool)),
}

impl Component for Filter {
    type Message = FilterMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(FilterMessage::ContextChanged))
            .unwrap();

        let ordering = ordering();
        context.properties_order_callback.emit(ordering.clone());

        Self {
            ordering,
            context,
            _listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FilterMessage::ContextChanged(context) => self.context = context,
            FilterMessage::FilterVisibilityChanged((name, selected)) => {
                let context = &self.context;
                let mut current_properties_order = context.properties_order.clone();
                if selected {
                    let mut new_order = self.ordering.clone();
                    for filter in &self.ordering {
                        if !context.properties_order.contains(filter) && *filter != name {
                            new_order.retain(|x| x != filter);
                        }
                    }

                    context.properties_order_callback.emit(new_order);

                    return true;
                } else {
                    current_properties_order.retain(|x| *x != name);
                }
                
                context.properties_order_callback.emit(current_properties_order);
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut ordering_properties: Vec<Html> = Vec::new();
        let properties_order = &self.context.properties_order;
        for ordering_property in &self.ordering {
            let callback = ctx.link().callback(move |(name, selected)| FilterMessage::FilterVisibilityChanged((name, selected)));

            let selected;
            if properties_order.contains(ordering_property) {
                selected = true;
            } else {
                selected = false;
            }

            ordering_properties.push(html! {
                <OrderingProperty name={ordering_property.clone()} selected={selected} callback={callback} />
            });
        }

        html! {
            <div class={classes!("filter")}>
                <div class={classes!("filter-content")}>
                    {ordering_properties}
                </div>
                <div 
                    class={classes!("resizer-right")}>
                </div>
            </div>
        }
    }
}

fn ordering() -> Vec<String> {
    let part_template = Part::default();
    let field_values_array = serde_json::to_string(&part_template);
    if let Ok(field_values_array) = field_values_array {
        let mut chars = field_values_array.chars();
        chars.next();
        chars.next_back();
        let field_values_array = format!("[{}]", chars.as_str());
        let field_values_array = field_values_array.replace(":", ",");
        let array: Result<Vec<Value>, serde_json::Error> = serde_json::from_str(&field_values_array);
        if let Ok(array) = array {
            let mut array: Vec<String> = array.iter().map(|x| x.to_string()).collect();
            let len = array.len();
            for i in 0..(len / 2) {
                array.remove(len - (i * 2) - 1);
            }

            for item in array.iter_mut() {
                let mut new_item = item.replace("\"", "");
                let mut chars: Vec<char> = new_item.chars().collect();
                let uppercase_char = chars[0].to_uppercase().nth(0);
                if let Some(uppercase_char) = uppercase_char {
                    chars[0] = uppercase_char;
                    new_item = chars.into_iter().collect();
                }
                *item = new_item.replace("_", " ");
            }

            return array;
        }
    }

    Vec::new()
}

pub struct OrderingProperty;

#[derive(Properties, Clone, PartialEq)]
pub struct OrderingPropertyProps {
    pub name: String,
    pub selected: bool,
    pub callback: Callback<(String, bool)>,
}

pub enum OrderingPropertyMessage {
    ChangeSelected(bool),
}

impl Component for OrderingProperty {
    type Message = OrderingPropertyMessage;
    type Properties = OrderingPropertyProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            OrderingPropertyMessage::ChangeSelected(selected) => { 
                let props = ctx.props();
                props.callback.emit((props.name.clone(), selected));
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_click = {
            let props = ctx.props().clone();
            ctx.link().callback(move |_| OrderingPropertyMessage::ChangeSelected(!props.selected))
        };

        html! {
            <div class={classes!("ordering-property")}>
                <input type="checkbox" onchange={on_click} checked={ctx.props().selected} />
                <h3>{ctx.props().name.clone()}</h3>
            </div>
        }
    }
}
