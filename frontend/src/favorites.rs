use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{app::AppContext, parts::Part};

pub struct Favorites {
    parts: Vec<Part>,
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
}

pub enum FavoritesMessage {
    ContextChanged(Rc<AppContext>),
    PopulateParts(Vec<Part>),
    SetSelected(String, bool),
    SetFavorite(String, bool),
}

impl Component for Favorites {
    type Message = FavoritesMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(FavoritesMessage::ContextChanged))
            .unwrap();

        let callback = ctx.link().callback(move |parts| { FavoritesMessage::PopulateParts(parts) });
        spawn_local(get_parts(context.clone(), callback));

        Self {
            parts: Vec::new(),
            context,
            _listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FavoritesMessage::ContextChanged(context) => self.context = context,
            FavoritesMessage::PopulateParts(mut parts) => {
                let selected_parts = &self.context.selected_parts;
                let favorited_parts = &self.context.favorites;
                for part in parts.iter_mut() {
                    if selected_parts.contains(&part.id) {
                        part.selected = true;
                    }

                    if favorited_parts.contains(&part.id) {
                        part.favorited = true;
                    }
                }

                self.parts = parts
            },
            FavoritesMessage::SetSelected(part_id, selected) => {
                let part = self.parts.iter_mut().find(|x| x.id == part_id);
                if let Some(part) = part {
                    part.selected = selected;
                    self.context.selected_parts_callback.emit((part.id.clone(), selected));
                }
            },
            FavoritesMessage::SetFavorite(part_id, favorited) => {
                let part = self.parts.iter_mut().find(|x| x.id == part_id);
                if let Some(part) = part {
                    part.favorited = favorited;
                    self.context.favorites_callback.emit((part.id.clone(), favorited));
                }
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback_selected = ctx.link().callback(move |(part_id, selected): (String, bool)| FavoritesMessage::SetSelected(part_id.clone(), !selected));
        let callback_favorite = ctx.link().callback(move |(part_id, selected): (String, bool)| FavoritesMessage::SetFavorite(part_id.clone(), !selected));

        let parts = self.parts.clone();
        if !parts.is_empty() {
            let html: Vec<Html> = parts.iter().map(|x| x.to_html(None, callback_selected.clone(), callback_favorite.clone())).collect();

            return html! {
                <div class={classes!("parts")}>
                    {html}
                </div>
            };
        }

        html! {
            <div class={classes!("comparison-empty")}>
                <h2>{"No favorites"}</h2> 
            </div>
        }
    }
}

async fn get_parts(context: Rc<AppContext>, callback: Callback<Vec<Part>>) {
    let mut result: Vec<Part> = Vec::new();
    let favorites = context.favorites.clone();
    for favorite in favorites {
        let part = context.get_part(favorite).await;
        if let Some(part) = part {
            result.push(part);
        }
    }

    callback.emit(result);
}
