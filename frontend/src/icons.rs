use yew::prelude::*;

pub struct SearchBar;

impl Component for SearchBar {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("search-bar-container")}>
                <div class={classes!("filter-icon")}>
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
