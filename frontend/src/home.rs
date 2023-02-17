use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::AppRoute;

#[function_component]
pub fn Home() -> Html {
    html! {
        <Redirect<AppRoute> to={AppRoute::Parts} />
    }
}
