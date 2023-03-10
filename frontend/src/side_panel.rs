use std::rc::Rc;

use common::{PartsCategory, traits::PartProperties};
use yew::prelude::*;

use crate::{app::AppContext, comparison::ComparisonContext};

#[derive(Clone, PartialEq)]
pub enum SidePanelConfig {
    Settings,
    Tabs,
}

pub struct SidePanel {
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
    comparison_context: Rc<ComparisonContext>,
    _comparison_listener: ContextHandle<Rc<ComparisonContext>>,
}

pub enum SidePanelMessage {
    ContextChanged(Rc<AppContext>),
    ComparisonContextChanged(Rc<ComparisonContext>),
    SetSelectedCategory(String),
}

#[derive(Properties, PartialEq, Clone)]
pub struct SidePanelProps {
    pub config: SidePanelConfig,
}

impl Component for SidePanel {
    type Message = SidePanelMessage;
    type Properties = SidePanelProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(SidePanelMessage::ContextChanged))
            .unwrap();

        let (comparison_context, _comparison_listener) = ctx
            .link()
            .context::<Rc<ComparisonContext>>(ctx.link().callback(SidePanelMessage::ComparisonContextChanged))
            .unwrap();

        Self {
            context,
            _listener,
            comparison_context,
            _comparison_listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SidePanelMessage::ContextChanged(context) => self.context = context,
            SidePanelMessage::ComparisonContextChanged(context) => self.comparison_context = context,
            SidePanelMessage::SetSelectedCategory(category) => self.context.selected_category_callback.emit(category),
        }

        true 
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let comparison_context = &self.comparison_context;
        let div = match &props.config {
            SidePanelConfig::Settings => {
                let settings = settings(PartsCategory::from_string(&self.context.selected_category));
                let settings: Vec<Html> = settings.iter().map(|x| {
                    html! {
                        <div class={classes!("settings")}>
                            <h2>{x}</h2>
                        </div>
                    }
                }).collect();

                html! {
                    <div
                        class={classes!("settings")}>
                        <h2>{"Settings"}</h2>
                        {settings}
                    </div>
                }
            },
            SidePanelConfig::Tabs => { 
                let mut categories: Vec<String> = Vec::new();
                for part in &comparison_context.parts {
                    let category = part.category_properties.to_string();
                    if !categories.contains(&category) {
                        categories.push(category);
                    }
                }

                let main_callback = ctx.link().callback(move |category| SidePanelMessage::SetSelectedCategory(category));

                let tabs: Vec<Html> = categories.iter().map(|x| {
                    let callback = {
                        let callback_cloned = main_callback.clone();
                        let category = x.clone();
                        Callback::from(move |_| {
                            callback_cloned.emit(category.clone());
                        })
                    };

                    let selected = x == &self.context.selected_category;

                    if selected {
                        html! {
                            <div 
                                onclick={callback}
                                class={classes!("tab", "selected")}>
                                <h2>{x}</h2>
                            </div>
                        }
                    } else {
                        html! {
                            <div 
                                onclick={callback}
                                class={classes!("tab")}>
                                <h2>{x}</h2>
                            </div>
                        }
                    }
                }).collect();

                html! {
                    <div
                        class={classes!("tabs")}>
                        {tabs}
                    </div>
                }
            },
        };

        html! {
            <div class={classes!("side-panel")}>
                {div}
                <div 
                    class={classes!("resizer-right")}>
                </div>
            </div>
        }
    }
}

fn settings(category: PartsCategory) -> Vec<String> {
    let category_field_map = category.to_string_vec();
    let mut category_field_names: Vec<String> = Vec::new();
    if let Ok(category_field_map) = category_field_map {
        for (key, _) in category_field_map {
            category_field_names.push(key);
        }
    }
    category_field_names
}
