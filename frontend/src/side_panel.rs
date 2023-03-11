use std::rc::Rc;

use yew::prelude::*;

use crate::{app::AppContext, comparison::ComparisonContext, filter::Property};

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
    FilterVisibilityChanged((String, bool)),
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
            SidePanelMessage::FilterVisibilityChanged((name, selected)) => {
                let context = &self.context;
                let mut properties_order = context.properties_order.clone();
                properties_order.insert(name, selected);
                context.properties_order_callback.emit(properties_order);
            },
        }

        true 
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let comparison_context = &self.comparison_context;
        let div = match &props.config {
            SidePanelConfig::Settings => {
                let callback = ctx.link().callback(move |(name, selected)| SidePanelMessage::FilterVisibilityChanged((name, selected)));
                let settings = &self.context.properties_order;
                let settings: Vec<Html> = settings.iter().map(|x| {
                    let name = x.0;
                    let selected = *x.1;
                    html! {
                        <Property name={name.to_owned()} selected={selected} callback={callback.clone()} />
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
