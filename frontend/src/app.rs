use std::rc::Rc;

use common::{GetPartProps, DBPart};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{content::{ContentPage, Content}, header::Header, Footer, parts::Part, connection::post_from_db};

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub content_page: ContentPage,
    pub content_page_callback: Callback<ContentPage>,
    pub selected_parts: Vec<String>,
    pub selected_parts_callback: Callback<(String, bool)>,
}

pub async fn get_part_with_callback(context: Rc<AppContext>, id: String, callback: Callback<Part>) {
    let part = context.get_part(id).await;
    if let Some(part) = part {
        callback.emit(part);
    }
}

pub async fn get_parts_with_callback(context: Rc<AppContext>, limit: u32, callback: Callback<Vec<Part>>) {
    let parts = context.get_parts(limit).await;
    if parts.len() > 0 {
        callback.emit(parts);
    }
}


impl AppContext {
    pub async fn get_part(&self, id: String) -> Option<Part> {
        let json = GetPartProps {
            id: Some(id),
            limit: 1,
        };
        
        let db_part: Option<DBPart> = post_from_db("http://127.0.0.1:8088/api/part", json).await;

        if let Some(db_part) = db_part {
            let part: Part = db_part.into();
            return Some(part);
        }

        None
    }

    pub async fn get_parts(&self, limit: u32) -> Vec<Part> {
        let json = GetPartProps {
            id: None,
            limit,
        };
        
        let db_parts: Option<Vec<DBPart>> = post_from_db("http://127.0.0.1:8088/api/part", json).await;

        if let Some(db_parts) = db_parts {
            let parts: Vec<Part> = db_parts.iter().map(|x| Part::from(x.clone())).collect();
            return parts;
        }

        Vec::new()
    }
}

pub struct App {
    app_context: Rc<AppContext>,
}

pub enum AppMessage {
    ChangeContentPage(ContentPage),
    UpdateSelectedPart(String, bool),
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let content_page_callback = ctx.link().callback(move |page| AppMessage::ChangeContentPage(page));
        let selected_parts_callback = ctx.link().callback(move |(part, selected)| AppMessage::UpdateSelectedPart(part, selected));
        
        let context = Rc::new(AppContext {
            content_page: ContentPage::Parts,
            content_page_callback,
            selected_parts: Vec::new(),
            selected_parts_callback,
        });

        Self { app_context: context }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let mut app_context = Rc::make_mut(&mut self.app_context);
        match msg {
            AppMessage::ChangeContentPage(page) => {
                app_context.content_page = page;
            },
            AppMessage::UpdateSelectedPart(part, selected) => {
                if selected {
                    app_context.selected_parts.push(part);
                } else {
                    app_context.selected_parts.retain(|x| *x != part);
                }
            },
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let app_context = &self.app_context;
        html! {
            <ContextProvider<Rc<AppContext>> context={app_context}>
                <BrowserRouter>
                    <div class={classes!("body")}>
                        <Header />
                        <Content />
                        <Footer />
                    </div>
                </BrowserRouter>
            </ContextProvider<Rc<AppContext>>>
        }
    }
}

#[derive(Routable, Clone, PartialEq)]
pub enum AppRoute {
    #[at("/")]
    Home,
    #[at("/part/:id")]
    Part { id: String },
    #[at("/parts")]
    Parts,
    #[at("/comparison")]
    Comparison,
    #[at("/create")]
    Create,
    #[not_found]
    #[at("/404")]
    NotFound,
}
