use agenda_parser::{Event, event::EventKind, location::Building};
use yew::prelude::*;
use std::{sync::atomic::{AtomicUsize, Ordering}};
use chrono::TimeZone;
use chrono_tz::Europe::Paris;
use wasm_bindgen::{prelude::*, JsCast};
use crate::settings::{SETTINGS, BuildingNaming};

lazy_static::lazy_static!{
    static ref ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Properties, Clone)]
pub struct EventCompProps {
    pub event: Event,
    pub day_start: u64,
}

impl PartialEq for EventCompProps {
    fn eq(&self, other: &Self) -> bool {
        self.event.start_unixtime == other.event.start_unixtime && self.event.end_unixtime == other.event.end_unixtime // TODO: add other fields
    }
}

pub struct EventComp {
    show_details: bool,
    on_click: Closure<dyn FnMut(web_sys::MouseEvent)>,
    ignore_next_event: bool,
    popup_id: String,
}

pub enum EventCompMsg {
    ToggleDetails,
}

impl Component for EventComp {
    type Message = EventCompMsg;
    type Properties = EventCompProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = format!("event-popup-{}", ID_COUNTER.fetch_add(1, Ordering::Relaxed));

        // Creates a closure called on click that will close the popup if the user clicked outside of it
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let id2 = id.clone();
        let link = ctx.link().clone();
        let on_click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let popup_el = document.get_element_by_id(&id2).unwrap();
            let rect = popup_el.get_bounding_client_rect();

            // Check the click was not inside the popup
            if (rect.y()..rect.y()+rect.height()).contains(&(event.client_y() as f64))
                && (rect.x()..rect.x()+rect.width()).contains(&(event.client_x() as f64))
            { return; }

            link.send_message(EventCompMsg::ToggleDetails);
        }) as Box<dyn FnMut(_)>);

        EventComp {
            show_details: false,
            on_click,
            ignore_next_event: false,
            popup_id: id,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EventCompMsg::ToggleDetails if self.show_details => {
                // We sometimes need a mechanism to ignore the next message. When the event listener is added, it instantly fires another ToggleDetails message which would close the popup instantly if not ignored
                if self.ignore_next_event {
                    self.ignore_next_event = false;
                    false
                } else {
                    self.show_details = false;

                    // Popup is already closed, so we can spare resources by disabling the event listener
                    web_sys::window().unwrap().remove_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();
                    true
                }
            },
            EventCompMsg::ToggleDetails => {
                self.show_details = true;

                // Popup is now opened so we must be ready to close it
                web_sys::window().unwrap().add_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();

                self.ignore_next_event = true;

                true
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sec_offset = ctx.props().event.start_unixtime.saturating_sub(ctx.props().day_start + 8 * 3600);
        let percent_offset = 100.0 / (44100.0) * sec_offset as f64;
        let percent_height = 100.0 / (44100.0) * (ctx.props().event.end_unixtime - ctx.props().event.start_unixtime) as f64;

        let name = match &ctx.props().event.kind {
            EventKind::Td(kind) => format!("TD: {}", kind),
            EventKind::Cm(kind) => format!("CM: {}", kind),
            EventKind::Tp(kind) => format!("TP: {}", kind),
            EventKind::Other(kind) => kind.to_string(),
        };
        
        let location = ctx.props().event.location.map(|location| {
            let building =  match SETTINGS.building_naming() {
                BuildingNaming::Short => match location.building {
                    Building::Magellan => "Ma",
                    Building::DumontDurville => "Du",
                },
                BuildingNaming::Long => match location.building {
                    Building::Magellan => "Magellan",
                    Building::DumontDurville => "Dumont Durville",
                },
            };

            format!("{} - {} - {} - {}", building, location.building_area, location.level, location.room_number)
        });
    
        let start = Paris.timestamp(ctx.props().event.start_unixtime as i64, 0);
        let end = Paris.timestamp(ctx.props().event.end_unixtime as i64, 0);
        let duration = (ctx.props().event.end_unixtime - ctx.props().event.start_unixtime) / 60;
        let groups = ctx.props().event.groups.iter().map(|g| format!("{:?}", g)).collect::<Vec<_>>().join(", ");
        
        // Specify font-size according event height
        let font_size = percent_height/8.;
        let font_size = if font_size > 1.2 { 1.2 } else { font_size };
        html! {
            <div
                style={format!("background-color: #98fb98; position: absolute; top: {}%; height: {}%; width: 100%;", percent_offset, percent_height)}
                class="event"
                onclick={ if !self.show_details { Some(ctx.link().callback(|_| EventCompMsg::ToggleDetails)) } else {None} } >

                <span class="name" style={format!("font-size: {}em",font_size)} >
                    { &name }
                </span>
                <span class="teacher" style={format!("font-size: {}em",font_size)}>
                    { ctx.props().event.teachers.join(", ") }
                </span>
                {if let Some(l) = &location {html! {<span style={format!("font-size: {}em",font_size)} >{l}</span>}} else {html!{}}}
                <div class="event-details" id={self.popup_id.clone()} style={if self.show_details {""} else {"display: none;"}} >
                    <div class="event-details-header">
                        <span>{ name }</span>
                    </div>
                    <div class="event-details-content">
                        <div>
                            <span class="bold">{"Début : "}</span>
                            {start.time().format("%Hh%M")}
                        </div>
                        <div>
                            <span class="bold">{"Fin : "}</span>
                            {end.time().format("%Hh%M")}
                        </div>
                        <div>
                            <span class="bold">{"Durée : "}</span>
                            {duration}{"min"}
                        </div>
                        <div>
                            <span class="bold">{ if ctx.props().event.groups.len() > 1 {{"Groupes : "}} else {{"Groupe : "}} }</span>
                            {groups}
                        </div>
                        <div>
                            <span class="bold">{"Professeur : "}</span>
                            {ctx.props().event.teachers.join(", ")}
                        </div>
                        <div>
                            <span class="bold">{"Salle : "}</span>
                            {location.unwrap_or_else(|| "Inconnue".to_string())}
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
