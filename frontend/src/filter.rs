use yew::prelude::*;

pub struct Filter;

impl Component for Filter {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("filter")}>
            </div>
        }
    }
}
