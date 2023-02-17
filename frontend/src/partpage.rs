use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use yew::{Component, Properties, Callback, html, ContextHandle};

use crate::{parts::Part, app::AppContext};

pub struct PartPage {
    pub part: Option<Part>,
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

#[derive(Properties, PartialEq, Clone)]
pub struct PartPageProperties {
    pub part_id: String,
}

pub enum PartPageMessages {
    ContextChanged(Rc<AppContext>),
    PopulatePart(Part),
}

impl Component for PartPage {
    type Message = PartPageMessages;
    type Properties = PartPageProperties;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(PartPageMessages::ContextChanged))
            .unwrap();

        {
            let part_id = ctx.props().part_id.clone();
            let callback = ctx.link().callback(move |part| { PartPageMessages::PopulatePart(part) });
            spawn_local(get_part_from_db(context.clone(), part_id.clone(), callback));
        }

        Self {
            part: None,
            context,
            _listener,
        }
    }

    fn update(&mut self, _: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PartPageMessages::PopulatePart(part) => self.part = Some(part),
            PartPageMessages::ContextChanged(context) => self.context = context,
        }

        true
    }

    fn view(&self, _: &yew::Context<Self>) -> yew::Html {
        if let Some(part) = &self.part {
            return html! {
                <h2>{part.name.clone()}</h2>
            }
        }

        html! {
            <h2>{ "Loading..." }</h2>
        }
    }
}

async fn get_part_from_db(context: Rc<AppContext>, part_id: String, callback: Callback<Part>) {
    let part = context.get_part(part_id).await.unwrap();
    callback.emit(part);
}
