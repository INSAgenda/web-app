use crate::prelude::*;

pub struct GiftComp {
    show_popup: bool,
}

pub enum GiftMsg {
    OpenGift,
}

impl Component for GiftComp {
    type Message = GiftMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            show_popup: true,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        // Handle message
        match msg {
            GiftMsg::OpenGift => {
                self.show_popup = false;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.show_popup {
            template_html!(
                "src/advent/gift/gift.html",
                show_popup = show_popup,
                onclick_gift = { ctx.link().callback(|_| GiftMsg::OpenGift) },
            )
        } else {
            html! {
                <div></div>
            }
        }
    }
}
