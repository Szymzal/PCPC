use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::AppRoute;

pub struct Header;

impl Component for Header {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("header")}>
                <Link<AppRoute> classes={classes!("link")} to={AppRoute::Parts}>{ "Parts" }</Link<AppRoute>>
                <Link<AppRoute> classes={classes!("link")} to={AppRoute::Comparison}>{ "Compare" }</Link<AppRoute>>
                <Link<AppRoute> classes={classes!("link")} to={AppRoute::Favorites}>{ "Favorites" }</Link<AppRoute>>
                <Link<AppRoute> classes={classes!("link")} to={AppRoute::Create}>{ "Create" }</Link<AppRoute>>
            </div>
        }
    }
}
