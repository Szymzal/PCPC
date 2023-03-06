use std::rc::Rc;

use yew::prelude::*;

use crate::app::AppContext;

#[derive(Clone, PartialEq)]
pub enum SidePanelConfig {
    Settings,
    Tabs,
}

pub struct SidePanel {
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

pub enum SidePanelMessage {
    ContextChanged(Rc<AppContext>),
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

        Self {
            context,
            _listener,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let div = match &props.config {
            SidePanelConfig::Settings => html! {
                <div
                    class={classes!("settings")}>

                </div>
            },
            SidePanelConfig::Tabs => html! {
                <div
                    class={classes!("tabs")}>

                </div>
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
