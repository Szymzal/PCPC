use std::rc::Rc;

use common::PartsCategory;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{app::AppContext, parts::Part, side_panel::{SidePanel, SidePanelConfig}};

pub struct Comparison {
    comparison_context: Rc<ComparisonContext>,
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

#[derive(Clone, PartialEq)]
pub struct ComparisonContext {
    pub parts: Vec<Part>,
}

pub enum ComparisonMessage {
    ContextChanged(Rc<AppContext>),
    PopulateParts(Vec<Part>),
}

impl Component for Comparison {
    type Message = ComparisonMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(ComparisonMessage::ContextChanged))
            .unwrap();

        let callback = ctx.link().callback(move |parts| ComparisonMessage::PopulateParts(parts));
        spawn_local(Comparison::get_parts(context.clone(), callback));

        let comparison_context = Rc::new(
            ComparisonContext {
                parts: Vec::new(),
            }
        );

        Self { 
            comparison_context,
            context,
            _listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ComparisonMessage::ContextChanged(context) => self.context = context,
            ComparisonMessage::PopulateParts(parts) => {
                let context = Rc::make_mut(&mut self.comparison_context);
                context.parts = parts;
            },
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut part_names = Vec::new();
        let comparison_context = &self.comparison_context;
        let comparison_parts = &comparison_context.parts;
        for part in comparison_parts {
            part_names.push(html! {
                <th>
                    {&part.name}
                </th>
            });
        }

        let default_part = Part {
            category_properties: PartsCategory::from_string(&self.context.selected_category),
            ..Default::default()
        };
        let default_part_string = default_part.get_properties_as_map();
        let mut properties: Vec<Html> = Vec::new();
        if let Ok(default_part_string) = default_part_string {
            for property in default_part_string.keys() {
                properties.push(get_property_from_parts(comparison_parts, property.to_string()));
            }
        }

        html! {
            <ContextProvider<Rc<ComparisonContext>> context={comparison_context}>
                <div class={classes!("comparison")}>
                    <SidePanel config={SidePanelConfig::Tabs} />
                    <table class={classes!("comparison-table")}>
                        <tr>
                            <th></th>
                            {part_names}
                        </tr>
                        {properties}
                    </table>
                </div>
            </ContextProvider<Rc<ComparisonContext>>>
        }
    }
}

fn get_property_from_parts(parts: &Vec<Part>, property: String) -> Html {
    let mut part_properties: Vec<Html> = Vec::new();
    part_properties.push(html! {
        <td>
            {&property}
        </td>
    });

    for part in parts {
        let properties = part.get_properties_as_map();
        if let Ok(properties) = properties {
            let property = properties.get(&property);
            if let Some(property) = property {
                part_properties.push(html! {
                    <td>
                        {property}
                    </td>
                });
            }
        }
    }

    html! {
        <tr>
            {part_properties}
        </tr>
    }
}

impl Comparison {
    async fn get_parts(context: Rc<AppContext>, callback: Callback<Vec<Part>>) {
        let mut parts: Vec<Part> = Vec::new();
        for selected_part_id in &context.selected_parts {
            let part = context.get_part(selected_part_id.to_owned()).await;
            if let Some(part) = part {
                parts.push(part);
            }
        }

        callback.emit(parts);
    }
}
