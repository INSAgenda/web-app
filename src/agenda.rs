use chrono::{NaiveDateTime, NaiveTime};

use crate::{prelude::*, api, slider};

fn format_day(day_name: Weekday, day: u32, month: u32) -> String {
    let month = t(match month {
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
    });

    let day_name = t(match day_name {
        Weekday::Mon => "Lundi",
        Weekday::Tue => "Mardi",
        Weekday::Wed => "Mercredi",
        Weekday::Thu => "Jeudi",
        Weekday::Fri => "Vendredi",
        Weekday::Sat => "Samedi",
        Weekday::Sun => "Dimanche",
    });

    format!("{} {} {}", day_name, day, month)
}
pub struct Agenda {
    selected_day: Date<chrono_tz::Tz>,
    events: Vec<RawEvent>,
    slider: Rc<RefCell<slider::SliderManager>>,
    announcements: Vec<AnnouncementDesc>,
    pub displayed_announcement: Option<AnnouncementDesc>,
    selected_event: Option<RawEvent>,
}

pub enum AgendaMsg {
    ScheduleSuccess(Vec<RawEvent>),
    ScheduleFailure(ApiError),
    Previous,
    Next,
    Goto {day: u32, month: u32, year: i32},
    SetSliderState(bool),
    SetSelectedEvent(Option<common::Event>),
    CloseAnnouncement,
    AnnouncementsSuccess(Vec<AnnouncementDesc>),
    Refresh,
}

pub fn refresh_events(agenda_link: Scope<Agenda>) {
    wasm_bindgen_futures::spawn_local(async move {
        match api::load_events().await {
            Ok(events) => agenda_link.send_message(AgendaMsg::ScheduleSuccess(events)),
            Err(e) => agenda_link.send_message(AgendaMsg::ScheduleFailure(e)),
        }
    });
}
#[derive(Properties, Clone)]
pub struct AgendaProps {
    pub app_link: Scope<App>,
}

impl PartialEq for AgendaProps {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl Component for Agenda {
    type Message = AgendaMsg;
    type Properties = AgendaProps;

    fn create(ctx: &Context<Self>) -> Self {
        let now = chrono::Local::now();
        let now = now.with_timezone(&Paris);

        // Update events
        let mut skip_event_loading = false;
        let mut events = Vec::new();
        if let Some((last_updated, cached_events)) = api::load_cached_events() {
            if last_updated > now.timestamp() - 3600*5 && !cached_events.is_empty() {
                skip_event_loading = true;
            }
            events = cached_events;
        }
        if !skip_event_loading {
            refresh_events(ctx.link().clone());
        }

        // Update announcements
        let mut skip_announcements_loading = false;
        let mut announcements = Vec::new();
        if let Some((last_updated, cached_announcements)) = api::load_cached_announcements() {
            if last_updated > now.timestamp() - 3600*12 && !cached_announcements.is_empty() {
                skip_announcements_loading = true;
            }
            announcements = cached_announcements;
        }
        let displayed_announcement = select_announcement(&announcements);
        if !skip_announcements_loading {
            let link2 = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::load_announcements().await {
                    Ok(events) => link2.send_message(AgendaMsg::AnnouncementsSuccess(events)),
                    Err(e) => e.handle_api_error(),
                }
            });
        }
        // Switch to next day if it's late or to monday if it's weekend
        let weekday = now.weekday();
        let curr_day = now.naive_local().date().and_hms(0, 0, 0);
        let has_event = has_event_on_day(&events, curr_day, Weekday::Sat);
        if now.hour() >= 19 || weekday == Weekday::Sun || (weekday == Weekday::Sat && !has_event) {
            let link2 = ctx.link().clone();
            spawn_local(async move {
                sleep(Duration::from_millis(500)).await;
                link2.send_message(AgendaMsg::Next);
            });
        }

        Self {
            events,
            selected_day: now.date(),
            slider: slider::SliderManager::init(ctx.link().clone(), -20 * (now.date().num_days_from_ce() - 730000)),
            announcements,
            displayed_announcement,
            selected_event: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AgendaMsg::ScheduleSuccess(events) => {
                self.events = events;
                true
            },
            AgendaMsg::ScheduleFailure(api_error) => {
                api_error.handle_api_error();
                match api_error {
                    ApiError::Known(error) if error.kind == "counter_too_low" => {
                        refresh_events(ctx.link().clone());
                    }
                    _ => {},
                }
                false
            },
            AgendaMsg::AnnouncementsSuccess(announcements) => {
                self.announcements = announcements;
                false // Don't think we should refresh display of the page because it would cause high inconvenience and frustration to the users
            },
            AgendaMsg::Previous => {
                let prev_week = NaiveDateTime::new(self.selected_day.naive_local(), NaiveTime::from_hms(0, 0, 0)) - chrono::Duration::days(7);
                if self.selected_day.weekday() != Weekday::Mon {
                    self.selected_day = self.selected_day.pred();
                } else if self.selected_day.weekday() == Weekday::Mon && !has_event_on_day(&self.events, prev_week, Weekday::Sat) {
                    self.selected_day = self.selected_day.pred().pred().pred();
                } else {
                    self.selected_day = self.selected_day.pred().pred();
                }
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));
                true
            },
            AgendaMsg::Next => {
                let now = NaiveDateTime::new(self.selected_day.naive_local(), NaiveTime::from_hms(0, 0, 0));
                if self.selected_day.weekday() == Weekday::Sat {
                    self.selected_day = self.selected_day.succ().succ();
                } else if self.selected_day.weekday() != Weekday::Fri {
                    self.selected_day = self.selected_day.succ();
                } else if self.selected_day.weekday() == Weekday::Fri && !has_event_on_day(&self.events, now, Weekday::Sat) {
                    self.selected_day = self.selected_day.succ().succ().succ();
                } else {
                    self.selected_day = self.selected_day.succ();
                }
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));
                true
            },
            AgendaMsg::Goto {day, month, year} => {
                self.selected_day = Paris.ymd(year, month, day);
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
            },
            AgendaMsg::CloseAnnouncement => update_close_announcement(self),
            AgendaMsg::SetSliderState(state) => {
                let mut slider = self.slider.borrow_mut();
                match state {
                    true => slider.enable(),
                    false => slider.disable(),
                }
                true
            },
            AgendaMsg::SetSelectedEvent(event) => {
                self.selected_event = event;
                true
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mobile = crate::slider::width() <= 1000;
        // Go on the first day of the week
        let mut current_day = self.selected_day;
        match mobile {
            true => current_day = current_day.pred().pred(),
            false => for _ in 0..self.selected_day.weekday().num_days_from_monday() {
                current_day = current_day.pred();
            },
        };

        // Check if there is room for the announcement on mobile
        let announcement = self.displayed_announcement.as_ref();
        let mut show_mobile_announcement = mobile && announcement.is_some();
        if show_mobile_announcement {
            let announcement_start = current_day.succ().succ().and_hms(18,30,0).timestamp() as u64;
            let announcement_end = current_day.succ().succ().and_hms(20,0,0).timestamp() as u64;
            let announcement_range = announcement_start..=announcement_end;

            match self.events.binary_search_by_key(&announcement_start, |e| e.start_unixtime) { // Check if an event starts exactly at that time.
                Ok(_) => {
                    show_mobile_announcement = false;
                },
                Err(mut idx) => {
                    if let Some(e) = self.events.get(idx) {
                        if announcement_range.contains(&(e.start_unixtime)) { // Check if the next event starts in the range
                            show_mobile_announcement = false;
                        } else {
                            idx -= 1;
                            while let Some(e) = self.events.get(idx) { // Check if a few previous events end in the range
                                if announcement_range.contains(&(e.end_unixtime)) {
                                    show_mobile_announcement = false;
                                    break;
                                }
                                if e.end_unixtime < announcement_start - 6*3600 { // Avoid backtracking too much, 6h is enough
                                    break;
                                }
                                idx -= 1;
                            }
                        }
                    }
                },
            };
        }
        let agenda_class = if show_mobile_announcement { "show-announcement" } else { "" };
        let announcement = announcement.map(|a| view_announcement(a, ctx));

        // Build each day and put events in them
        let mut days = Vec::new();
        let mut day_names = Vec::new();
        for d in 0..6 {
            let mut events = Vec::new();

            // Iterate over events, starting from the first one that starts during the current day
            let day_start = current_day.and_hms(0,0,0).timestamp() as u64;
            let mut idx = match self.events.binary_search_by_key(&day_start, |e| e.start_unixtime) {
                Ok(idx) => idx,
                Err(idx) => idx,
            };
            while let Some(e) = self.events.get(idx) {
                if e.start_unixtime > day_start + 24*3600 {
                    break;
                }
                events.push(html!{
                    <EventComp
                        day_of_week={d}
                        event={e.clone()}
                        day_start={current_day.and_hms(0,0,0).timestamp() as u64}
                        agenda_link={ctx.link().clone()}
                        show_announcement={false}>
                    </EventComp>
                });
                idx += 1;
            }

            let day_style = match mobile {
                true => format!("position: absolute; left: {}%;", (current_day.num_days_from_ce()-730000) * 20),
                false => String::new(),
            };

            day_names.push(html! {
                <span id={if current_day == self.selected_day {"selected-day"} else {""}}>
                    { format_day(current_day.weekday(), current_day.day(), current_day.month()) }
                </span>
            });
            days.push(html! {
                <div class="day" style={day_style}>
                    { events }
                </div>
            });

            current_day = current_day.succ();
        }

        html! {
            <>
            <header>
                <a id="header-logo" href="/agenda">
                <img src="/assets/logo/logo.svg" alt="INSAgenda logo"/> 
                <h1 id="header-name">{"INSAgenda"}</h1>
                </a>
                <button id="settings-button" onclick={ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Settings))}/>
            </header>
            <main id="agenda-main" class={agenda_class}>
            <div id="agenda">
                <div id="agenda-hours">
                    <span>{"08:00"}</span>
                    <span>{"09:45"}</span>
                    <span>{"11:30"}</span>
                    <span>{"13:15"}</span>
                    <span>{"15:00"}</span>
                    <span>{"16:45"}</span>
                    if !show_mobile_announcement {<span>{"18:30"}</span>}
                </div>
                <div id="agenda-main-part">
                    <div id="agenda-top">
                        <a id="agenda-arrow-left" onclick={ctx.link().callback(|_| AgendaMsg::Previous)}>
                            <div></div>
                        </a>
                        { day_names }
                        <a id="agenda-arrow-right" onclick={ctx.link().callback(|_| AgendaMsg::Next)}>
                            <div></div>
                        </a>
                    </div>
                    <div id="day-container-scope">
                        <div id="day-container" style={if mobile {Some(format!("position: relative; right: {}%", 100 * (self.selected_day.num_days_from_ce() - 730000)))} else {None}}>
                            { days }
                        </div>
                    </div>
                </div>
            </div>
    
            <Popup
                event={self.selected_event.clone()}
                agenda_link={ctx.link().clone()}>
            </Popup>
            if mobile && show_mobile_announcement {
                if let Some(announcement) = announcement {
                    { announcement }
                }
            }
        </main>
            </>
        }
    }
}
