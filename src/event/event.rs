use crate::prelude::*;

lazy_static::lazy_static!{
    static ref ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Properties, Clone)]
pub struct EventCompProps {
    pub event: RawEvent,
    pub day_start: u64,
    pub agenda_link: AgendaLink,
    pub week_day: u8,
    pub comment_counts: Rc<CommentCounts>,
    pub seen_comment_counts: Rc<CommentCounts>,
}

impl PartialEq for EventCompProps {
    fn eq(&self, other: &Self) -> bool {
        !COLORS_CHANGED.load(Ordering::Relaxed)
            && self.event == other.event
            && self.day_start == other.day_start
            && self.week_day == other.week_day
            && self.comment_counts.get(&self.event.eid) == other.comment_counts.get(&other.event.eid)
            && self.seen_comment_counts.get(&self.event.eid) == other.seen_comment_counts.get(&other.event.eid)
    }
}

pub trait HackTraitEventFormat {
    fn format_name(&self) -> String;
    fn format_location(&self) -> Option<String>;
    fn format_time(&self) -> String;
}
impl HackTraitEventFormat for RawEvent {
    fn format_name(&self) -> String {
        let summary = &self.summary;
        match self.kind {
            Some(EventKind::Td) => format!("TD: {summary}"),
            Some(EventKind::Tp) => format!("TP: {summary}"),
            Some(EventKind::Cm) => format!("CM: {summary}"),
            None => summary.clone(),
        }
    }

    fn format_location(&self) -> Option<String> {
        self.location.as_ref().map(|location| {
            match location {
                Location::Parsed { building, building_area, level, room_number } => {
                    let building = match building {
                        Building::Magellan => "Ma",
                        Building::DumontDurville => "Du",
                        Building::Bougainville => "Bo",
                        Building::Darwin => "Da",
                    };
                    format!("{} - {} - {} - {}", building, building_area, level, room_number)
                }
                Location::Unparsed(location) => location.clone(),
            }
        })
    }

    fn format_time(&self) -> String {
        let start = Paris.timestamp_opt(self.start_unixtime as i64, 0).unwrap();
        let end = Paris.timestamp_opt(self.end_unixtime as i64, 0).unwrap();
        format!("{} - {}", start.time().format("%Hh%M"), end.time().format("%Hh%M"))
    }
}

pub struct EventComp {}

impl Component for EventComp {
    type Message = ();
    type Properties = EventCompProps;

    fn create(_ctx: &Context<Self>) -> Self { Self {} }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let name = ctx.props().event.format_name();
        let location = ctx.props().event.format_location();
        let bg_color = COLORS.get(&ctx.props().event.summary);

        // Calculate position
        let day_sec_count = 43200.0;
        let sec_offset = ctx.props().event.start_unixtime.saturating_sub(ctx.props().day_start + 8 * 3600);
        let percent_offset = 100.0 / day_sec_count * sec_offset as f64;
        if ctx.props().event.start_unixtime >= ctx.props().event.end_unixtime {
            log!("Event {} in {:?}  ends before it starts", name, location);
            return html!{};
        }
        let percent_height = 100.0 / day_sec_count * (ctx.props().event.end_unixtime - ctx.props().event.start_unixtime) as f64;

        // Count comments
        let opt_comment_count = ctx.props().comment_counts.get(&ctx.props().event.eid).copied();
        let seen_comment_count = ctx.props().seen_comment_counts.get(&ctx.props().event.eid).copied().unwrap_or_default();
        let comment_count = ctx.props().comment_counts.get(&ctx.props().event.eid).copied().unwrap_or_default();
        let seen = seen_comment_count >= comment_count;

        // Render
        let eid = ctx.props().event.eid.clone(); // FIXME: what if eid contains slashes and stuff?
        let onclick = ctx.props().agenda_link.callback(move |_| AgendaMsg::AppMsg(AppMsg::SetPage(Page::Event { eid: eid.clone() } )));
        template_html!(
            "src/event/event.html",
            teachers = { ctx.props().event.teachers.join(", ")},
            opt_location = location,
            bg_color = {bg_color.clone()},
            ...
        )
    }
}
