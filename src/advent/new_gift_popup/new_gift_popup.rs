use crate::prelude::*;

#[derive(Properties, Clone)]
pub struct NewGiftPopupProps {
    pub agenda_link: AgendaLink,
}

impl PartialEq for NewGiftPopupProps {
    fn eq(&self, _other: &Self) -> bool { true }
}


pub struct NewGiftPopupComp {
    show_popup: bool,
}

pub enum NewGiftPopuptMsg {
    OpenGift,
}

impl Component for NewGiftPopupComp {
    type Message = NewGiftPopuptMsg;
    type Properties = NewGiftPopupProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            show_popup: true,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        // Handle message
        match msg {
            NewGiftPopuptMsg::OpenGift => {
                self.show_popup = false;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.show_popup {
            template_html!(
                "src/advent/new_gift_popup/new_gift_popup.html",
            )
        } else {
            html! {
                <div></div>
            }
        }
    }
}
