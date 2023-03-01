use yew::prelude::*;

mod content;
pub mod app;
mod header;
mod parts;
mod comparison;
mod home;
mod create;
mod partpage;
mod connection;
mod filter;
mod icons;
mod rating;

#[function_component]
fn Footer() -> Html {
    html! {
        <div class={classes!("footer")}>

        </div>
    }
}
