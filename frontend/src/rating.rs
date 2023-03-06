use yew::prelude::*;

pub struct Rating;

#[derive(Properties, PartialEq, Clone)]
pub struct RatingProps {
    pub rating: f32,
}

impl Component for Rating {
    type Message = ();
    type Properties = RatingProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();
        let percent = props.rating / 5.0 * 100.0;

        html! {
            <div 
                class={classes!("part_rating")}
                style={
                    format!("--percent: {}%", percent)
                }
            >
            {"★★★★★"}
            </div>
        }
    }
}
