use agenda_parser::Event;
use yew::prelude::*;
use std::{rc::Rc, cell::Cell};

pub struct EventGlobalData {
    opened_event: Cell<Option<Rc<ComponentLink<EventComp>>>>
}

impl Default for EventGlobalData {
    fn default() -> Self {
        EventGlobalData {
            opened_event: Cell::new(None)
        }
    }
}

#[derive(Properties, Clone)]
pub struct EventCompProp {
    pub event: Event,
    pub day_start: u64,
    pub global: Rc<EventGlobalData>,
}

pub struct EventComp {
    link: Rc<ComponentLink<Self>>,
    global: Rc<EventGlobalData>,
    day_start: u64,
    event: Event,
    show_details: bool,
}

pub enum EventCompMsg {
    ToggleDetails,
    Replaced,
}

impl Component for EventComp {
    type Message = EventCompMsg;
    type Properties = EventCompProp;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        EventComp {
            event: props.event,
            day_start: props.day_start,
            global: props.global,
            link: Rc::new(link),
            show_details: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            EventCompMsg::ToggleDetails if self.show_details => {
                self.show_details = false;
                self.global.opened_event.set(None);
                true
            },
            EventCompMsg::Replaced => {
                self.show_details = false;
                true
            },
            EventCompMsg::ToggleDetails => {
                self.show_details = true;
                if let Some(old_link) = self.global.opened_event.take() {
                    old_link.send_message(EventCompMsg::Replaced);
                }
                self.global.opened_event.set(Some(self.link.clone()));
                true
            },
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.event = props.event;
        self.show_details = false;
        self.day_start = props.day_start;
        self.global = props.global;
        true
    }

    fn view(&self) -> Html {
        let sec_offset = self.event.start_unixtime - (self.day_start + 8 * 3600);
        let percent_offset = 100.0 / (44100.0) * sec_offset as f64;
        let percent_height = 100.0 / (44100.0) * (self.event.end_unixtime - self.event.start_unixtime) as f64;

        let name = match &self.event.kind {
            agenda_parser::event::EventKind::Td(kind) => format!("TD: {}", kind),
            agenda_parser::event::EventKind::Cm(kind) => format!("CM: {}", kind),
            agenda_parser::event::EventKind::Tp(kind) => format!("TP: {}", kind),
            agenda_parser::event::EventKind::Other(kind) => kind.to_string(),
        };

        let location = self.event.location.map(|location| location.to_string());

        html! {
            <div style=format!("background-color: #98fb98; position: absolute; top: {}%; height: {}%;", percent_offset, percent_height) class="event" onclick=self.link.callback(|_| EventCompMsg::ToggleDetails)>
                <span class="name">{ name }</span>
                <span class="teacher">{ self.event.teachers.join(", ") }</span>
                {if let Some(l) = location {html! {<span>{l}</span>}} else {html!{}}}
                <div class="event-details" style=if self.show_details {""} else {"display: none;"}>
                    <div class="event-details-header">
                        <span>{"01h00 - Lundi 3 janvier"}</span>
                    </div>
                    <div class="event-details-content">
                        {"ALLO"}
                    </div>
                </div>
            </div>
        }
    }
}
