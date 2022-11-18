use crate::prelude::*;

lazy_static::lazy_static!{
    static ref ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Properties, Clone)]
pub struct EventCompProps {
    pub event: RawEvent,
    pub day_start: u64,
    pub show_announcement: bool,
    pub agenda_link: yew::html::Scope<Agenda>,
    pub day_of_week: u8,
}

impl PartialEq for EventCompProps {
    fn eq(&self, other: &Self) -> bool {
        !COLORS_CHANGED.load(Ordering::Relaxed)
            && self.event == other.event
            && self.day_start == other.day_start
            && self.show_announcement == other.show_announcement
            && self.day_of_week == other.day_of_week
    }
}

pub struct EventComp {}

impl Component for EventComp {
    type Message = ();
    type Properties = EventCompProps;

    fn create(_ctx: &Context<Self>) -> Self { Self {} }

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

        // Render
        let event1 = ctx.props().event.clone();
        let day_of_week = ctx.props().day_of_week;
        template_html!(
            "templates/components/event.html",
            onclick = { ctx.props().agenda_link.callback(move |_| AgendaMsg::SetSelectedEvent(Some((day_of_week, event1.clone())))) },
            teachers = { ctx.props().event.teachers.join(", ")},
            opt_location = location, bg_color2 = bg_color, ...
        )
    }
}
