use crate::{prelude::*, slider::width, agenda::AgendaMsg};

lazy_static::lazy_static!{
    static ref ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Properties, Clone)]
pub struct EventCompProps {
    pub event: RawEvent,
    pub day_start: u64,
    pub show_announcement: bool,
    pub agenda_link: yew::html::Scope<crate::Agenda>,
    pub day_of_week: u8,
}

impl PartialEq for EventCompProps {
    fn eq(&self, other: &Self) -> bool {
        !COLORS_CHANGED.load(Ordering::Relaxed) && self.event.start_unixtime == other.event.start_unixtime && self.event.end_unixtime == other.event.end_unixtime // TODO: add other fields
    }
}

pub struct EventComp {
    popup_displayed: bool,
    last_click_timestamp: i64,
    popup_id: String,
}



impl Component for EventComp {
    type Message = ();
    type Properties = EventCompProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = format!("event-popup-{}", ID_COUNTER.fetch_add(1, Ordering::Relaxed));

        EventComp {
            popup_displayed: false,
            last_click_timestamp: 0,
            popup_id: id,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Format title
        let summary = &ctx.props().event.summary;
        let name = match &ctx.props().event.kind {
            Some(EventKind::Td) => format!("TD: {summary}"),
            Some(EventKind::Tp) => format!("TP: {summary}"),
            Some(EventKind::Cm) => format!("CM: {summary}"),
            None => summary.clone(),
        };
        let bg_color = COLORS.get(&ctx.props().event.summary);
        
        // Format location
        let location = ctx.props().event.location.as_ref().map(|location| {
            match location {
                Location::Parsed { building, building_area, level, room_number } => {
                    let building = match SETTINGS.building_naming() {
                        BuildingNaming::Short => match building {
                            Building::Magellan => "Ma",
                            Building::DumontDurville => "Du",
                            Building::Bougainville => "Bo",
                            Building::Darwin => "Da",
                        },
                        BuildingNaming::Long => match building {
                            Building::Magellan => "Magellan",
                            Building::DumontDurville => "Dumont Durville",
                            Building::Bougainville => "Bougainville",
                            Building::Darwin => "Darwin",
                        },
                    };
                    format!("{} - {} - {} - {}", building, building_area, level, room_number)
                }
                Location::Unparsed(location) => location.clone(),
            }
        });

        // Calculate position
        let day_sec_count = match ctx.props().show_announcement {
            false => 43200.0,
            true => 43200.0 - 6300.0,
        };
        let sec_offset = ctx.props().event.start_unixtime.saturating_sub(ctx.props().day_start + 8 * 3600);
        let percent_offset = 100.0 / day_sec_count * sec_offset as f64;
        if ctx.props().event.start_unixtime >= ctx.props().event.end_unixtime {
            log!("Event {} in {:?}  ends before it starts", name, location);
            return html!{};
        }
        let percent_height = 100.0 / day_sec_count * (ctx.props().event.end_unixtime - ctx.props().event.start_unixtime) as f64;        

        let event1 = ctx.props().event.clone();
        html! {<>
            <div
                style={format!("background-color: {}80; border-left: 0.3rem solid {}; top: {}%; height: {}%;", bg_color.clone(), bg_color.clone(), percent_offset, percent_height)}
                class="event" >
                <div class="event-container"  onclick={if !self.popup_displayed{ Some(ctx.props().agenda_link.callback(move |_| AgendaMsg::SetSelectedEvent(Some(event1.clone())) ))} else { None } } >
                    <span class="name" >
                        { &name }
                    </span>
                    <span class="teacher">
                        { ctx.props().event.teachers.join(", ") }
                    </span>
                    if let Some(l) = &location { <span class="location" >{l}</span>}
                </div>
            </div>
        </>}
    }
}
