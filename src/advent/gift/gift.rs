use crate::prelude::*;

use super::gifts::CollectedGifts;

#[derive(Properties, Clone)]
pub struct AdventProps {
    pub agenda_link: AgendaLink,
}

impl PartialEq for AdventProps {
    fn eq(&self, other: &Self) -> bool { true }
}

pub struct GiftComp {
    day: u8,
    collected: bool,
}

pub enum GiftMsg {
    OpenGift,
}

impl Component for GiftComp {
    type Message = GiftMsg;
    type Properties = AdventProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let local_storage = window().local_storage().unwrap().unwrap();
        let collected_gifts = match local_storage.get_item("collected_gifts").unwrap() {
            Some(json) => CollectedGifts::from_json(&json).unwrap_or_default(),
            None => CollectedGifts::default(),
        };

        

        Self {
            day: 0,
            collected: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        // Handle message
        match msg {
            GiftMsg::OpenGift => {
                self.collected = false;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if !self.collected {
            template_html!(
                "src/advent/gift/gift.html",
                onclick_gift = { ctx.link().callback(|_| GiftMsg::OpenGift) },
            )
        } else {
            html! {
                <div></div>
            }
        }
    }
}
