use std::rc::Rc;

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
                    <img 
                        src="https://cdn-icons-png.flaticon.com/512/622/622669.png" 
                        alt="Search icon" 
                    />
                    <input type="text" />
                </div>
            </div>
        }
    }
}
