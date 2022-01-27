use agenda_parser::{Event, event::EventKind, location::Building};
use yew::prelude::*;
use std::{rc::Rc, cell::Cell};
use chrono::TimeZone;
use chrono_tz::Europe::Paris;
use crate::settings::{SETTINGS, BuildingNaming};

pub struct EventGlobalData {
    opened_event: Cell<Option<yew::html::Scope<EventComp>>>
}

impl Default for EventGlobalData {
    fn default() -> Self {
        EventGlobalData {
            opened_event: Cell::new(None)
        }
    }
}

#[derive(Properties, Clone)]
pub struct EventCompProps {
    pub event: Event,
    pub day_start: u64,
    pub global: Rc<EventGlobalData>,
}

impl PartialEq for EventCompProps {
    fn eq(&self, other: &Self) -> bool {
        self.event.start_unixtime == other.event.start_unixtime && self.event.end_unixtime == other.event.end_unixtime // TODO: add other fields
    }
}

pub struct EventComp {
    show_details: bool,
}

pub enum EventCompMsg {
    ToggleDetails,
    Replaced,
}

impl Component for EventComp {
    type Message = EventCompMsg;
    type Properties = EventCompProps;

    fn create(_ctx: &Context<Self>) -> Self {
        EventComp {
            show_details: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EventCompMsg::ToggleDetails if self.show_details => {
                self.show_details = false;
                ctx.props().global.opened_event.set(None);
                true
            },
            EventCompMsg::Replaced => {
                self.show_details = false;
                true
            },
            EventCompMsg::ToggleDetails => {
                self.show_details = true;
                if let Some(old_link) = ctx.props().global.opened_event.take() {
                    old_link.send_message(EventCompMsg::Replaced);
                }
                ctx.props().global.opened_event.set(Some(ctx.link().clone()));
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
            
            <div style={format!("background-color: #98fb98; position: absolute; top: {}%; height: {}%;", percent_offset, percent_height)} class="event" onclick={ ctx.link().callback(|_| EventCompMsg::ToggleDetails) } >
                <span class="name" style={format!("font-size: {}em",font_size)}>{ &name }</span>
                <span class="teacher" style={format!("font-size: {}em",font_size)}>{ ctx.props().event.teachers.join(", ") }</span>
                {if let Some(l) = location {html! {<span style={format!("font-size: {}em",font_size)} >{l}</span>}} else {html!{}}}
                <div class="event-details" style={if self.show_details {""} else {"display: none;"}}>
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
                    </div>
                </div>
            </div>
        }
    }
}
