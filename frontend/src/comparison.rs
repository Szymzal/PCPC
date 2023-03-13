use std::{rc::Rc, collections::HashMap};

use common::PartsCategory;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{app::AppContext, parts::{Part, format_property}, side_panel::{SidePanel, SidePanelConfig}};

pub struct Comparison {
    comparison_context: Rc<ComparisonContext>,
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
    config: Option<SidePanelConfig>,
}

#[derive(Clone, PartialEq)]
pub struct ComparisonContext {
    pub parts: Vec<Part>,
}

pub enum ComparisonMessage {
    ContextChanged(Rc<AppContext>),
    PopulateParts(Vec<Part>),
    ChangeConfig(SidePanelConfig),
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
            config: Some(SidePanelConfig::Tabs),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ComparisonMessage::ContextChanged(context) => self.context = context,
            ComparisonMessage::PopulateParts(parts) => {
                let context = Rc::make_mut(&mut self.comparison_context);
                context.parts = parts;
            },
            ComparisonMessage::ChangeConfig(new_config) => {
                if let Some(config) = &self.config {
                    if config == &new_config {
                        self.config = None;
                        return true;
                    }
                }

                self.config = Some(new_config);
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut part_names = Vec::new();
        let comparison_context = &self.comparison_context;
        let mut comparison_parts = comparison_context.parts.clone();
        if !comparison_parts.is_empty() {
            comparison_parts.retain(|x| x.category_properties.to_string() == self.context.selected_category);
            for part in &mut *comparison_parts {
                part_names.push(html! {
                    <th>
                        {&part.name}
                    </th>
                });
            }

            let mut properties: Vec<Html> = Vec::new();
            let selected_category = PartsCategory::from_string(&self.context.selected_category);
            let template_part = Part {
                category_properties: selected_category,
                ..Default::default()
            };
            let template_properties = template_part.get_properties_as_map().unwrap_or(HashMap::new());
            for (property, visible) in &self.context.properties_order {
                if *visible && template_properties.contains_key(property) {
                    properties.push(get_property_from_parts(&comparison_parts, property.to_string()));
                }
            }

            let tabs_callback = ctx.link().callback(|_| ComparisonMessage::ChangeConfig(SidePanelConfig::Tabs));
            let settings_callback = ctx.link().callback(|_| ComparisonMessage::ChangeConfig(SidePanelConfig::Settings));

            let config;
            if let Some(stored_config) = &self.config {
                config = stored_config.clone();
            } else {
                // Placeholder
                config = SidePanelConfig::Tabs;
            }

            let side_panel = self.config.is_some();

            return html! {
                <ContextProvider<Rc<ComparisonContext>> context={comparison_context}>
                    <div class={classes!("comparison")}>
                        if side_panel {
                            <SidePanel config={config} />
                        }
                        <table class={classes!("comparison-table")}>
                            <tr>
                                <th class={classes!("buttons")}>
                                    <div 
                                        onclick={tabs_callback}
                                        class={classes!("comparison-button")}>
                                        <h5>{"Tabs"}</h5>
                                    </div>
                                    <div 
                                        onclick={settings_callback}
                                        class={classes!("comparison-button")}>
                                        <h5>{"Settings"}</h5>
                                    </div>
                                </th>
                                {part_names}
                            </tr>
                            {properties}
                        </table>
                    </div>
                </ContextProvider<Rc<ComparisonContext>>>
            };
        }

        html! { 
            <div class={classes!("comparison-empty")}>
                <h2>{"Add parts to compare"}</h2> 
            </div>
        }
    }
}

fn get_property_from_parts(parts: &Vec<Part>, property: String) -> Html {
    let mut part_properties: Vec<Html> = Vec::new();
    part_properties.push(html! {
        <th>
            {&property}
        </th>
    });

    for part in parts {
        let properties = part.get_properties_as_map();
        if let Ok(properties) = properties {
            let first_property = properties.get(&property);
            if let Some(first_property) = first_property {
                let first_property = format_property(first_property.to_owned());

                let mut different = false;
                for second_part in parts {
                    let properties = second_part.get_properties_as_map();
                    if let Ok(properties) = properties {
                        let second_property = properties.get(&property);
                        if let Some(second_property) = second_property {
                            let second_property = format_property(second_property.to_owned());

                            if second_property != first_property {
                                different = true;
                                break;
                            }
                        }
                    }
                }

                let result = match different {
                    true => {
                        html! {
                            <td class={classes!("different")}>
                                {&first_property}
                            </td>
                        }
                    },
                    false => {
                        html! {
                            <td>
                                {&first_property}
                            </td>
                        }
                    },
                };

                part_properties.push(result);
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
