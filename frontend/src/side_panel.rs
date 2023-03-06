use std::rc::Rc;

use log::info;
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
        }

        true 
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let comparison_context = &self.comparison_context;
        let div = match &props.config {
            SidePanelConfig::Settings => {
                html! {
                    <div
                        class={classes!("settings")}>
                        <h2>{"Settings"}</h2>
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

                let tabs: Vec<Html> = categories.iter().map(|x| {
                    html! {
                        <div class={classes!("tab")}>
                            <h2>{x}</h2>
                        </div>
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
