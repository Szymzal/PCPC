use std::rc::Rc;

use yew::prelude::*;
use yew_router::prelude::*;

use crate::{parts::Parts, app::{AppContext, AppRoute}, comparison::Comparison, home::Home, create::CreatePart, favorites::Favorites};

#[derive(Clone, Copy, PartialEq)]
pub enum ContentPage {
    Parts,
    Comparison,
}

pub struct Content {
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

pub enum ContentMessages {
    ContextChanged(Rc<AppContext>),
}

impl Component for Content {
    type Message = ContentMessages;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(ContentMessages::ContextChanged))
            .unwrap();

        Self { context, _listener }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ContentMessages::ContextChanged(context) => {
                self.context = context;
                true
            },
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("content")}>
                <Switch<AppRoute> render={switch} />
            </div>
        }
    }
}

fn switch(routes: AppRoute) -> Html {
    match routes {
        AppRoute::Home => html! { <Home /> },
        AppRoute::Parts => html! { <Parts /> },
        AppRoute::Comparison => html! { <Comparison /> },
        AppRoute::Create => html! { <CreatePart /> },
        AppRoute::Favorites => html! { <Favorites /> },
        AppRoute::NotFound => html! { <h1>{ "404" }</h1> },
    }
}
