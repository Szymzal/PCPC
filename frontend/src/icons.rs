use std::rc::Rc;

use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::app::AppContext;

pub struct SearchBar {
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

pub enum SearchBarMessage {
    ContextChanged(Rc<AppContext>),
}

impl Component for SearchBar {
    type Message = SearchBarMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(SearchBarMessage::ContextChanged))
            .unwrap();

        Self {
            context,
            _listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SearchBarMessage::ContextChanged(context) => self.context = context,
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let callback = {
            let context = self.context.clone();
            Callback::from(move |_| {
                context.filter_visibility_callback.emit(!context.filter_visibility);
            })
        };

        let input_callback = self.context.search_term_callback.clone();
        let input = Callback::from(move |input_event: InputEvent| {
            let event: Event = input_event.dyn_into().unwrap_throw();
            let event_target = event.target().unwrap_throw();
            let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
            let value = target.value();
            input_callback.emit(value);
        });

        html! {
            <div class={classes!("search-bar-container")}>
                <div 
                    class={classes!("filter-icon")}
                    onclick={callback}
                >
                    <img 
                        src="https://img.icons8.com/fluency-systems-regular/256/empty-filter.png" 
                        alt="Filter icon" 
                    />
                </div>
                <div class={classes!("search-bar")}>
                    <input type="text" value={self.context.search_term.to_owned()} oninput={input} placeholder={"Search"} />
                </div>
            </div>
        }
    }
}
