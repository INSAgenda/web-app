use crate::{prelude::*, slider::width};

#[derive(Clone, PartialEq)]
pub enum PopupState {
    Opened { week_day: u8, event: RawEvent, popup_size: Option<usize> },
    Closing { week_day: u8, event: RawEvent, popup_size: Option<usize> },
    Closed,
}

impl PopupState {
    pub fn as_option(&self) -> Option<(u8, &RawEvent, Option<usize>)> {
        match self {
            PopupState::Opened { week_day, event, popup_size } => Some((*week_day, event, *popup_size)),
            PopupState::Closing { week_day, event, popup_size } => Some((*week_day, event, *popup_size)),
            PopupState::Closed => None,
        }
    }
}

pub struct Popup {}

pub enum PopupMsg {
    SaveColors,
}

#[derive(Properties, Clone)]
pub struct PopupProps {
    pub event: RawEvent,
    pub week_day: u8,
    pub agenda_link: Scope<Agenda>,
}

impl PartialEq for PopupProps {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event && self.week_day == other.week_day
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
            PopupMsg::SaveColors => {
                let mobile = width() <= 1000;
                let document = window().doc();
                let el = document.get_element_by_id("popup-color-input").unwrap();
                let background_color = el.dyn_into::<HtmlInputElement>().unwrap().value();

                COLORS.set(&ctx.props().event.summary, background_color); 

                // We need to set this so that other events know that they have to refresh
                COLORS_CHANGED.store(true, Ordering::Relaxed);

                if !mobile {
                    ctx.props().agenda_link.send_message(AgendaMsg::Refresh);
                }
                
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let week_day = ctx.props().week_day;
        let event = ctx.props().event.clone();
        let onclick_close = ctx.props().agenda_link.callback(move |_| AgendaMsg::AppMsg(AppMsg::SetPage(Page::Popup(PopupState::Closing { week_day, event: event.clone(), popup_size: None }))));
        let event_color = COLORS.get(&ctx.props().event.summary);
        let summary = &ctx.props().event.summary;
        let name = ctx.props().event.format_name();
        let opt_location = ctx.props().event.format_location();
        template_html!(
            "src/popup/popup.html",
            teachers = {ctx.props().event.teachers.join(", ")},
            time = {ctx.props().event.format_time()},
            name = {&name},
            onclick_close = {onclick_close.clone()},
            onclick_save = {ctx.link().callback(|_| PopupMsg::SaveColors)},
            opt_location = {&opt_location},
            event_color = {event_color.clone()},
            ...
        )
    }
}
