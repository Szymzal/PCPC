use std::rc::Rc;

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
        let mut parts = Vec::new();
        let comparison_context = &self.comparison_context;
        for part in &comparison_context.parts {
            parts.push(html! {
                <>
                    {&part.name}
                    <br/>
                </>
            });
        }

        html! {
            <ContextProvider<Rc<ComparisonContext>> context={comparison_context}>
                <div class={classes!("comparison")}>
                    <SidePanel config={SidePanelConfig::Tabs} />
                    <h2>{"Selected:"}</h2>
                    <h2>{parts}</h2>
                </div>
            </ContextProvider<Rc<ComparisonContext>>>
        }
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
