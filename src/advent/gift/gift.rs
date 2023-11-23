use crate::prelude::*;
use chrono::Local;

const KEY: &str = "last_click_date";

pub struct GiftComp {
    show_popup: bool,
}

impl Component for GiftComp {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            show_popup: true,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let today = Local::now().naive_local().num_days_from_ce();

        if self.show_popup {
            template_html!(
                "src/advent/gift/gift.html",
                show_popup = show_popup,
            )
        } else {
            html! {
                <div></div>
            }
        }
    }
}
