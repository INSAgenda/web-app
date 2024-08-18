use crate::{prelude::*, slider};

fn format_day(day_name: Weekday, day: u32) -> String {
    let day_name = t(match day_name {
        Weekday::Mon => "Lundi",
        Weekday::Tue => "Mardi",
        Weekday::Wed => "Mercredi",
        Weekday::Thu => "Jeudi",
        Weekday::Fri => "Vendredi",
        Weekday::Sat => "Samedi",
        Weekday::Sun => "Dimanche",
    });

    format!("{} {}", day_name, day)
}

pub struct Agenda {
    selected_day: NaiveDate,
    slider: Rc<RefCell<slider::SliderManager>>,
}

pub enum AgendaMsg {
    Previous,
    Next,
    Goto{ day: u32, month: u32, year: i32 },
    Refresh,
    AppMsg(Box<AppMsg>),
}

#[derive(Properties, Clone)]
pub struct AgendaProps {
    pub app_link: AppLink,
    pub events: Rc<Vec<RawEvent>>,
    #[prop_or_default]
    pub profile_src: Option<String>,
    pub user_info: Rc<Option<UserInfo>>,
    pub comment_counts: Rc<CommentCounts>,
    pub seen_comment_counts: Rc<CommentCounts>,
    pub friends: Rc<Option<FriendLists>>,
    pub colors: Rc<Colors>,
}

impl PartialEq for AgendaProps {
    fn eq(&self, other: &Self) -> bool {
        self.events == other.events
            && self.user_info == other.user_info
            && self.comment_counts == other.comment_counts
            && self.seen_comment_counts == other.seen_comment_counts
            && self.friends == other.friends
            && self.colors == other.colors
    }
}

impl Component for Agenda {
    type Message = AgendaMsg;
    type Properties = AgendaProps;

    fn create(ctx: &Context<Self>) -> Self {
        let now = chrono::Local::now();
        let now = now.with_timezone(&Paris);

        // Switch to next day if it's late or to monday if it's weekend
        let weekday = now.weekday();
        let has_event = has_event_on_day(&ctx.props().events, now.date_naive(), Weekday::Sat);
        if now.hour() >= 19 || weekday == Weekday::Sun || (weekday == Weekday::Sat && !has_event) {
            let link2 = ctx.link().clone();
            spawn_local(async move {
                sleep(Duration::from_millis(500)).await;
                link2.send_message(AgendaMsg::Next);
            });
        }

        Self {
            selected_day: now.date_naive(),
            slider: slider::SliderManager::init(ctx.link().clone(), -20 * (now.date_naive().num_days_from_ce() - 730000))
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AgendaMsg::Previous => {
                let day_prev_week = self.selected_day - chrono::Duration::days(7);
                if self.selected_day.weekday() != Weekday::Mon {
                    self.selected_day -= chrono::Duration::days(1);
                } else if self.selected_day.weekday() == Weekday::Mon && !has_event_on_day(&ctx.props().events, day_prev_week, Weekday::Sat) {
                    self.selected_day -= chrono::Duration::days(3);
                } else {
                    self.selected_day -= chrono::Duration::days(2);
                }
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));
                true
            },
            AgendaMsg::Next => {
                let day_this_week = self.selected_day;
                if self.selected_day.weekday() == Weekday::Sat {
                    self.selected_day += chrono::Duration::days(2);
                } else if self.selected_day.weekday() == Weekday::Fri && !has_event_on_day(&ctx.props().events, day_this_week, Weekday::Sat) {
                    self.selected_day += chrono::Duration::days(3);
                } else {
                    self.selected_day += chrono::Duration::days(1);
                }
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));
                
                true
            },
            AgendaMsg::Goto {day, month, year} => {
                if let Some(new_selected_day) = NaiveDate::from_ymd_opt(year, month, day) {
                    self.selected_day = new_selected_day;
                }
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));
                true
            }
            AgendaMsg::Refresh => {
                let window = window();
                match Reflect::get(&window.doc(), &JsValue::from_str("reflectTheme")) {
                    Ok(reflect_theme) => {
                        let reflect_theme: Function = match reflect_theme.dyn_into(){
                            Ok(reflect_theme) => reflect_theme,
                            Err(e) => {
                                log!("Failed to convert reflect theme: {:?}", e);
                                return true;
                            }
                        };
                    
                        Reflect::apply(&reflect_theme, &window.doc(), &Array::new()).expect("Failed to call reflectTheme");
                    }
                    Err(_) => log!("reflectTheme not found")
                }
                true
            }
            AgendaMsg::AppMsg(msg) => {
                ctx.props().app_link.send_message(*msg);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let opt_profile_src = ctx.props().profile_src.as_ref().cloned();
        let screen_width = crate::slider::width();
        let mobile = screen_width <= 1000;
        
        // Go on the first day of the week
        let mut current_day = self.selected_day;
        match mobile {
            true => current_day -= chrono::Duration::days(2),
            false => for _ in 0..self.selected_day.weekday().num_days_from_monday() {
                current_day -= chrono::Duration::days(1);
            },
        };

        // Build each day and put events in them
        let mut days = Vec::new();
        let mut day_names = Vec::new();
        for d in 0..6 {
            let day_start = Paris.from_local_datetime(&current_day.and_hms_opt(0,0,0).unwrap()).unwrap().timestamp() as u64;

            // Iterate over events, starting from the first one that starts during the current day
            let mut idx = match ctx.props().events.binary_search_by_key(&day_start, |e| e.start_unixtime) {
                Ok(idx) => idx,
                Err(idx) => idx,
            };
            let mut events = Vec::new();
            while let Some(e) = ctx.props().events.get(idx) {
                if e.start_unixtime > day_start + 24*3600 {
                    break;
                }
                events.push(e);
                idx += 1;
            }

            // Find overlapping events
            let mut overlapping_events = Vec::new();
            for (i, e) in events.iter().enumerate() {
                let e_range = e.start_unixtime..e.end_unixtime;
                for (i2, e2) in events.iter().enumerate() {
                    if e == e2 { continue }
                    let e2_range = e2.start_unixtime..e2.end_unixtime;
                    if e2_range.contains(&e_range.start) || e2_range.contains(&e_range.end.saturating_sub(1)) {
                        overlapping_events.push(i);
                        overlapping_events.push(i2);
                    }
                }
            }
            overlapping_events.sort();
            overlapping_events.dedup();
            
            // Generate event components
            let mut event_comps = Vec::new();
            for (i, e) in events.into_iter().enumerate() {
                event_comps.push(html!{
                    <EventComp
                        week_day={d}
                        event={e.clone()}
                        day_start={day_start}
                        agenda_link={ctx.link().clone()}
                        vertical_offset={overlapping_events.iter().position(|i2| i == *i2).map(|i| (i, overlapping_events.len())).unwrap_or((0, 1))}
                        comment_counts={Rc::clone(&ctx.props().comment_counts)}
                        seen_comment_counts={Rc::clone(&ctx.props().seen_comment_counts)}
                        colors={Rc::clone(&ctx.props().colors)}>
                    </EventComp>
                });
                idx += 1;
            }

            // Generate day styles
            let mut day_style = String::new();
            if mobile {
                day_style.push_str(&format!("position: absolute; left: {}%;", (current_day.num_days_from_ce()-730000) * 20));
            }

            let day_name = match SETTINGS.calendar() {
                CalendarKind::Gregorian => format_day(current_day.weekday(), current_day.day()),
                CalendarKind::Republican => match RepublicanDateTime::try_from(current_day) {
                    Ok(datetime) => match datetime.num_month() {
                        13 => datetime.decade_day().to_string(),
                        _ => format!("{} {}", datetime.decade_day(), datetime.day()),
                    },
                    Err(_) => String::from("invalid date"),
                },
            };
            day_names.push(html! {
                <span id={if current_day == self.selected_day {"selected-day"} else {""}}>
                    { day_name }
                </span>
            });
            days.push(html! {
                <div class="day" id={format!("day{d}")} style={day_style}>
                    { event_comps }
                </div>
            });

            current_day += chrono::Duration::days(1);
        }

        let calendar = html! {
            <Calendar
                agenda_link={ctx.link().clone()}
                day={self.selected_day.day()}
                month={self.selected_day.month()}
                year={self.selected_day.year()} />
        };
        
        let day_container_style = if mobile {
            format!("right: {}%", 100 * (self.selected_day.num_days_from_ce() - 730000))
        } else {
            String::new()
        };

        template_html!(
            "src/agenda/agenda.html",
            onclick_rick = {ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Rick))},
            onclick_previous = {ctx.link().callback(|_| AgendaMsg::Previous)},
            onclick_next = {ctx.link().callback(|_| AgendaMsg::Next)},
            republican = {SETTINGS.calendar() == CalendarKind::Republican},
            moyeninsage = {SETTINGS.theme() == Theme::MoyenInsage},
            ...
        )
    }
}
