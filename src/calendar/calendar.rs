use crate::{prelude::*, slider::width};

#[derive(Clone, Properties)]
pub struct CalendarProps {
    pub agenda_link: Scope<Agenda>,
    pub day: u32,
    pub month: u32,
    pub year: i32,
}

impl PartialEq for CalendarProps {
    fn eq(&self, other: &Self) -> bool {
        self.day == other.day && self.month == other.month && self.year == other.year
    }
}

pub enum Msg {
    Next,
    Previous,
    Goto { day: u32, month: u32, year: i32 },
    TriggerFold,
}

pub struct Calendar {
    folded: bool,
    on_click: Closure<dyn FnMut(web_sys::MouseEvent)>,
}

impl Component for Calendar {
    type Message = Msg;
    type Properties = CalendarProps;

    fn create(ctx: &Context<Self>) -> Self {
        let doc = window().doc();
        let link = ctx.link().clone();
        let on_click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let Some(calendar_el) = doc.get_element_by_id("calendar-content") else {return};
            let rect = calendar_el.get_bounding_client_rect();

            let (mx, my) = (event.client_x() as f64, event.client_y() as f64);
            let (rx, ry) = (rect.x(), rect.y());
            let (rw, rh) = (rect.width(), rect.height());

            // Check the click was inside the calendar
            if ((ry..ry+rh).contains(&my) && (rx..rx+rw).contains(&mx)) || (my <= ry) { return; }

            link.send_message(Msg::TriggerFold);
        }) as Box<dyn FnMut(_)>);
        Calendar {
            folded: true,
            on_click,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Next if !self.folded => {
                let mut month = ctx.props().month;
                let mut year = ctx.props().year;
                if month == 12 {
                    month = 1;
                    year += 1;
                } else {
                    month += 1;
                }
                ctx.props().agenda_link.send_message(AgendaMsg::Goto { day: 1, month, year });
            },
            Msg::Previous if !self.folded => {
                let mut month = ctx.props().month;
                let mut year = ctx.props().year;
                if month == 1 {
                    month = 12;
                    year -= 1;
                } else {
                    month -= 1;
                }
                let day = NaiveDate::from_ymd_opt(year, (month % 12) + 1, 1).unwrap().pred_opt().unwrap().day();
                ctx.props().agenda_link.send_message(AgendaMsg::Goto { day, month, year });
            },
            Msg::Next => {
                let next_week = NaiveDate::from_ymd_opt(ctx.props().year, ctx.props().month, ctx.props().day).unwrap() + chrono::Duration::days(7);
                ctx.props().agenda_link.send_message(AgendaMsg::Goto {
                    day: next_week.day(),
                    month: next_week.month(),
                    year: next_week.year()
                });
            },
            Msg::Previous => {
                let previous_week = NaiveDate::from_ymd_opt(ctx.props().year, ctx.props().month, ctx.props().day).unwrap() - chrono::Duration::days(7);
                ctx.props().agenda_link.send_message(AgendaMsg::Goto {
                    day: previous_week.day(),
                    month: previous_week.month(),
                    year: previous_week.year()
                });
            }
            Msg::Goto { day, month, year } => {
                ctx.props().agenda_link.send_message(AgendaMsg::Goto {day, month, year});
            },
            Msg::TriggerFold => {
                self.folded = !self.folded;
                match self.folded {
                    true => window().remove_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap(),
                    false => window().add_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap(),
                };
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let selected: chrono::NaiveDate = NaiveDate::from_ymd_opt(ctx.props().year, ctx.props().month, ctx.props().day).expect("Invalid date");
        let today = Local::now().date_naive();

        let display_month = match SETTINGS.calendar() {
            CalendarKind::Gregorian => {
                format!("{} {}", t(match ctx.props().month {
                    1 => "Janvier",
                    2 => "Février",
                    3 => "Mars",
                    4 => "Avril",
                    5 => "Mai",
                    6 => "Juin",
                    7 => "Juillet",
                    8 => "Août",
                    9 => "Septembre",
                    10 => "Octobre",
                    11 => "Novembre",
                    12 => "Décembre",
                    _ => unreachable!(),
                }), ctx.props().year)
            },
            CalendarKind::Republican => {
                let selected: calendrier::Date = selected.try_into().expect("Could not convert date");
                format!("{} {}", selected.month(), selected.year())
            }
        };

        let mut week_iter = Vec::new();
        let mut cases_iter = Vec::new();
        match SETTINGS.calendar() {
            CalendarKind::Gregorian => {
                let first_day = NaiveDate::from_ymd_opt(ctx.props().year, ctx.props().month, 1).unwrap();
                let last_day = NaiveDate::from_ymd_opt(ctx.props().year, (ctx.props().month % 12) + 1, 1).unwrap().pred_opt().unwrap();
        
                let mut calendar_cases = Vec::new();
                for _ in 0..first_day.weekday().number_from_monday() - 1 {
                    calendar_cases.push(html! {
                        <span class="calendar-case" onclick={ctx.link().callback(|_| Msg::Previous)}></span>
                    });
                }
                for day in 1..=last_day.day() {
                    let (month, year) = (ctx.props().month, ctx.props().year);
                    let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
                    let id = if day==ctx.props().day {Some("calendar-case-selected")} else if date==today {Some("calendar-case-today")} else {None};
                    calendar_cases.push(html! {
                        <span class="calendar-case" id={id} onclick={ctx.link().callback(move |_| Msg::Goto {day,month,year})}>{day.to_string()}</span>
                    });
                }
                while calendar_cases.len() % 7 != 0 {
                    calendar_cases.push(html! {
                        <span class="calendar-case" onclick={ctx.link().callback(|_| Msg::Next)}></span>
                    });
                }

                for week in 1..=calendar_cases.len()/7 {
                    week_iter.push(week);
                    cases_iter.push(calendar_cases.drain(0..7).collect::<Vec<_>>());
                }
            },
            CalendarKind::Republican => {
                let selected_republican: calendrier::DateTime = selected.try_into().expect("Could not convert date");
                let first_day = calendrier::DateTime::from_ymd_hms0(selected_republican.year0(), selected_republican.num_month0(), 0, selected_republican.hour(), selected_republican.minute(), selected_republican.second());

                for decade in 0..3 {
                    let mut decade_cases = Vec::new();
                    for decade_day in 0..10 {
                        let day0 = decade*10 + decade_day;
                        let date = first_day.clone() + chrono::Duration::days(day0);
                        let gregorian: chrono::NaiveDate = date.try_into().unwrap();
                        let id = if gregorian == selected {Some("calendar-case-selected")} else if gregorian == today {Some("calendar-case-today")} else {None};

                        decade_cases.push(html! {
                            <span
                                class="calendar-case"
                                id={id}
                                onclick={ctx.link().callback(move |_| Msg::Goto {
                                    day: gregorian.day0()+1,
                                    month: gregorian.month0()+1,
                                    year: gregorian.year()
                                })}
                            >
                                {(day0+1).to_string()}
                            </span>
                        });
                    }
                    week_iter.push(decade as usize+1);
                    cases_iter.push(decade_cases);
                }
            }
        }

        let mobile = width() <= 1000;
        let show_arrows = !mobile || !self.folded;

        template_html! {
            "src/calendar/calendar.html",
            onclick_previous = {ctx.link().callback(|_| Msg::Previous)},
            onclick_fold = {ctx.link().callback(|_| Msg::TriggerFold)},
            onclick_next = {ctx.link().callback(|_| Msg::Next)},
            week_iter = {week_iter.iter()},
            cases_iter = {cases_iter.iter()},
            is_folded = {self.folded},
            republican = {SETTINGS.calendar() == CalendarKind::Republican},
            ...
        }
    }
}
