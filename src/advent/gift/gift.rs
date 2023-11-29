use crate::prelude::{*, gifts::GiftList};
use super::gifts::CollectedGifts;

lazy_static::lazy_static!{
    static ref GIFT_LIST: GiftList = GiftList::from_json(include_str!("../gifts.json")).unwrap();
}

const START_DAY: i32 = 25;

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

        let today = Local::now().naive_local().num_days_from_ce() - START_DAY;
        let today = if today > u8::MAX as i32 {
            0
        } else {
            today as u8
        };
        
        let collected = collected_gifts.is_collected(today as i32);
        Self {
            day: today,
            collected,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        // Handle message
        match msg {
            GiftMsg::OpenGift => {
                self.collected = true;
                let local_storage = window().local_storage().unwrap().unwrap();
                let mut collected_gifts = CollectedGifts::from_local_storage();
                collected_gifts.collect(self.day);
                local_storage.set_item("collected_gifts", &collected_gifts.to_json()).unwrap();
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
