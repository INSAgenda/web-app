use crate::{prelude::*, slider::width};

pub struct Popup {}

pub enum PopupMsg {
    Close,
    SaveColors,
}

#[derive(Properties, Clone)]
pub struct PopupProps {
    pub event: Rc<RawEvent>,
    pub week_day: u8,
    pub agenda_link: Scope<Agenda>,
}

impl PartialEq for PopupProps {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event
    }
}

impl Component for Popup {
    type Message = PopupMsg;
    type Properties = PopupProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PopupMsg::Close => {
                ctx.props().agenda_link.send_message(AgendaMsg::SetSelectedEvent(None));
                false
            }
            PopupMsg::SaveColors => {
                //ctx.props().agenda_link.send_message(AgendaMsg::SaveColors);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick_close = ctx.link().callback(|_| PopupMsg::Close);
        template_html!(
            "templates/components/popup.html",
            teachers = {ctx.props().event.teachers.join(", ")},
            onclick_close = {onclick_close.clone()},
        )
    }
}
