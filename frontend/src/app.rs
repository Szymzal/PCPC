use std::{rc::Rc, collections::HashMap};

use common::{GetPartProps, DBPart, PartsCategory};
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{content::{ContentPage, Content}, header::Header, parts::Part, connection::post_from_db, filter::ordering};

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub content_page: ContentPage,
    pub content_page_callback: Callback<ContentPage>,
    pub selected_parts: Vec<String>,
    pub selected_parts_callback: Callback<(String, bool)>,
    pub properties_order: HashMap<String, bool>,
    pub properties_order_callback: Callback<HashMap<String, bool>>,
    pub selected_category: String,
    pub selected_category_callback: Callback<String>,
    pub filter_visibility: bool,
    pub filter_visibility_callback: Callback<bool>,
    pub favorites: Vec<String>,
    pub favorites_callback: Callback<(String, bool)>,
    pub search_term: String,
    pub search_term_callback: Callback<String>,
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
    mouse_event_selected: Option<HtmlDivElement>,
}

pub enum AppMessage {
    ChangeContentPage(ContentPage),
    UpdateSelectedPart(String, bool),
    SetMouseEventSelected(Option<MouseEvent>),
    UpdateSizeOfSelectedElement(MouseEvent),
    OrderPropertiesChange(HashMap<String, bool>),
    SetSelectedCategory(String),
    SetFilterVisibility(bool),
    UpdateFavorite((String, bool)),
    UpdateSearchTerm(String),
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let content_page_callback = ctx.link().callback(move |page| AppMessage::ChangeContentPage(page));
        let selected_parts_callback = ctx.link().callback(move |(part, selected)| AppMessage::UpdateSelectedPart(part, selected));
        let properties_order_callback = ctx.link().callback(move |new_ordering_properties| AppMessage::OrderPropertiesChange(new_ordering_properties));
        let filter_visibility_callback = ctx.link().callback(move |filter_visibility| AppMessage::SetFilterVisibility(filter_visibility));
        let selected_category_callback = ctx.link().callback(move |selected_category| AppMessage::SetSelectedCategory(selected_category));
        let favorites_callback = ctx.link().callback(move |(id, favorite)| AppMessage::UpdateFavorite((id, favorite)));
        let search_term_callback = ctx.link().callback(move |search_term| AppMessage::UpdateSearchTerm(search_term));

        let mut properties_order: HashMap<String, bool> = HashMap::new();
        for category in PartsCategory::get_all_variats() {
            let map = ordering(PartsCategory::from_string(&category));
            properties_order.extend(map);
        }

        let context = Rc::new(AppContext {
            content_page: ContentPage::Parts,
            content_page_callback,
            selected_parts: Vec::new(),
            selected_parts_callback,
            properties_order,
            properties_order_callback,
            selected_category: PartsCategory::default().to_string(),
            selected_category_callback,
            filter_visibility: true,
            filter_visibility_callback,
            favorites: Vec::new(),
            favorites_callback,
            search_term: "".to_string(),
            search_term_callback,
        });

        Self { 
            app_context: context,
            mouse_event_selected: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let mut app_context = Rc::make_mut(&mut self.app_context);
        match msg {
            AppMessage::ChangeContentPage(page) => app_context.content_page = page,
            AppMessage::UpdateSelectedPart(part, selected) => {
                if selected {
                    app_context.selected_parts.push(part);
                } else {
                    app_context.selected_parts.retain(|x| *x != part);
                }
            },
            AppMessage::SetMouseEventSelected(event) => {
                if let Some(event) = event {
                    let target = event.clone().target();
                    let element = target.and_then(|t| t.dyn_into::<HtmlDivElement>().ok());
                    if let Some(element) = element {
                        let classes = element.class_list();
                        if !classes.contains("resizer-right") {
                            return true;
                        }

                        let element = element.parent_element().and_then(|e| e.dyn_into::<HtmlDivElement>().ok());
                        if let Some(element) = element {
                            self.mouse_event_selected = Some(element);
                            return true;
                        }
                    }
                }

                self.mouse_event_selected = None;
            },
            AppMessage::UpdateSizeOfSelectedElement(event) => {
                if let Some(selected) = &self.mouse_event_selected {
                    let width = selected.client_width();
                    let style = selected.style();
                    let new_width = width + event.movement_x().clone();
                    style.set_property("width", format!("{}px", new_width).as_str()).unwrap();
                }
            },
            AppMessage::OrderPropertiesChange(properties_order) => app_context.properties_order = properties_order,
            AppMessage::SetFilterVisibility(filter_visibility) => app_context.filter_visibility = filter_visibility,
            AppMessage::SetSelectedCategory(selected_category) => app_context.selected_category = selected_category,
            AppMessage::UpdateFavorite((id, favorite)) => {
                if favorite {
                    app_context.favorites.push(id);
                } else {
                    app_context.favorites.retain(|x| *x != id);
                }
            },
            AppMessage::UpdateSearchTerm(search_term) => app_context.search_term = search_term,
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let app_context = &self.app_context;
        let callback_mouse_down = ctx.link().callback(move |event| AppMessage::SetMouseEventSelected(Some(event)));
        let callback_mouse_up = ctx.link().callback(move |_| AppMessage::SetMouseEventSelected(None));
        let callback = ctx.link().callback(move |event| AppMessage::UpdateSizeOfSelectedElement(event));

        html! {
            <ContextProvider<Rc<AppContext>> context={app_context}>
                <BrowserRouter>
                    <div 
                        class={classes!("body")}
                        onmousedown={callback_mouse_down}
                        onmouseup={callback_mouse_up.clone()}
                        onmouseleave={callback_mouse_up.clone()}
                        onmousemove={callback}
                    >
                        <Header />
                        <Content />
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
    #[at("/favorites")]
    Favorites,
    #[not_found]
    #[at("/404")]
    NotFound,
}
