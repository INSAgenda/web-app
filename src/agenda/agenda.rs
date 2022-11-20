use crate::{prelude::*, slider::{self, width}};

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
    selected_day: Date<chrono_tz::Tz>,
    slider: Rc<RefCell<slider::SliderManager>>,
    pub displayed_announcement: Option<AnnouncementDesc>,
    popup: PopupState,
    counter: AtomicUsize,
}

pub enum AgendaMsg {
    Previous,
    Next,
    Goto{ day: u32, month: u32, year: i32 },
    OpenPopup{ week_day: u8, event: RawEvent },
    ClosePopup,
    CloseAnnouncement,
    Refresh,
    PushColors
}

#[derive(Properties, Clone)]
pub struct AgendaProps {
    pub app_link: Scope<App>,
    pub user_info: Rc<Option<UserInfo>>,
    pub events: Rc<Vec<RawEvent>>,
    pub announcements: Rc<Vec<AnnouncementDesc>>,
}

impl PartialEq for AgendaProps {
    fn eq(&self, other: &Self) -> bool {
        !COLORS_CHANGED.load(Ordering::Relaxed) && self.user_info == other.user_info && self.events == other.events
    }
}

impl Component for Agenda {
    type Message = AgendaMsg;
    type Properties = AgendaProps;

    fn create(ctx: &Context<Self>) -> Self {
        let now = chrono::Local::now();
        let now = now.with_timezone(&Paris);

        // Select announcement
        let displayed_announcement = select_announcement(&ctx.props().announcements, &ctx.props().user_info.clone());

        // Trigger color sync when page is closed
        let link = ctx.link().clone();
        let unload = Closure::wrap(Box::new(move |_: web_sys::Event| {
            link.send_message(AgendaMsg::PushColors);
        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("unload", unload.as_ref().unchecked_ref()).unwrap();
        unload.forget();

        // Get colors
        crate::COLORS.fetch_colors(ctx.props().app_link.clone());

        // Auto-push colors every 15s if needed
        let link = ctx.link().clone();
        let push_colors = Closure::wrap(Box::new(move || {
            link.send_message(AgendaMsg::PushColors);
        }) as Box<dyn FnMut()>);
        if let Err(e) = window().set_interval_with_callback_and_timeout_and_arguments(push_colors.as_ref().unchecked_ref(), 1000*15, &Array::new()) {
            sentry_report(JsValue::from(&format!("Failed to set timeout: {:?}", e)));
        }
        push_colors.forget();

        // Switch to next day if it's late or to monday if it's weekend
        let weekday = now.weekday();
        let curr_day = now.naive_local().date().and_hms(0, 0, 0);
        let has_event = has_event_on_day(&ctx.props().events, curr_day, Weekday::Sat);
        if now.hour() >= 19 || weekday == Weekday::Sun || (weekday == Weekday::Sat && !has_event) {
            let link2 = ctx.link().clone();
            spawn_local(async move {
                sleep(Duration::from_millis(500)).await;
                link2.send_message(AgendaMsg::Next);
            });
        }
        
        Self {
            selected_day: now.date(),
            slider: slider::SliderManager::init(ctx.link().clone(), -20 * (now.date().num_days_from_ce() - 730000)),
            displayed_announcement,
            popup: PopupState::Closed,
            counter: AtomicUsize::new(0),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AgendaMsg::Previous => {
                let prev_week = NaiveDateTime::new(self.selected_day.naive_local(), NaiveTime::from_hms(0, 0, 0)) - chrono::Duration::days(7);
                if self.selected_day.weekday() != Weekday::Mon {
                    self.selected_day = self.selected_day.pred();
                } else if self.selected_day.weekday() == Weekday::Mon && !has_event_on_day(&ctx.props().events, prev_week, Weekday::Sat) {
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
                } else if self.selected_day.weekday() == Weekday::Fri && !has_event_on_day(&ctx.props().events, now, Weekday::Sat) {
                    self.selected_day = self.selected_day.succ().succ().succ();
                } else {
                    self.selected_day = self.selected_day.succ();
                }
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));
                
                true
            },
            AgendaMsg::Goto {day, month, year} => {
                self.selected_day = Paris.ymd(year, month, day);
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
            },
            AgendaMsg::CloseAnnouncement => update_close_announcement(self),
            AgendaMsg::OpenPopup { week_day, event } => {
                self.slider.borrow_mut().disable();
                let mut popup_size = None;
                if let PopupState::Opened { popup_size: Some(previous_size), .. } | PopupState::Closing { popup_size: Some(previous_size), .. } = self.popup {
                    popup_size = Some(previous_size);
                } else if let Some(day_el) = window().doc().get_element_by_id("day0") {
                    let rect = day_el.get_bounding_client_rect();
                    popup_size = Some((width() as f64 - rect.width() - 2.0 * rect.left()) as usize)
                }
                self.popup = PopupState::Opened { week_day, event: Rc::new(event), popup_size };
                spawn_local(async move {
                    window().doc().body().unwrap().set_attribute("style", "overflow: hidden").unwrap();
                    sleep(Duration::from_millis(500)).await;
                    window().doc().body().unwrap().remove_attribute("style").unwrap();
                });
                true
            },
            AgendaMsg::ClosePopup => {
                match self.popup {
                    PopupState::Opened { week_day, ref event, popup_size } => {
                        self.popup = PopupState::Closing { week_day, event: Rc::clone(event), popup_size };
                        let link = ctx.link().clone();
                        spawn_local(async move {
                            window().doc().body().unwrap().set_attribute("style", "overflow: hidden").unwrap();
                            sleep(Duration::from_millis(500)).await;
                            link.send_message(AgendaMsg::ClosePopup);
                            window().doc().body().unwrap().remove_attribute("style").unwrap();
                        });
                        true
                    },
                    PopupState::Closing { .. } => {
                        self.popup = PopupState::Closed;
                        self.slider.borrow_mut().enable();
                        true
                    },
                    PopupState::Closed => false,
                }
            }
            AgendaMsg::PushColors => {
                crate::COLORS.push_colors();
                false
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let screen_width = crate::slider::width();
        let mobile = screen_width <= 1000;
        
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

            match ctx.props().events.binary_search_by_key(&announcement_start, |e| e.start_unixtime) { // Check if an event starts exactly at that time.
                Ok(_) => {
                    show_mobile_announcement = false;
                },
                Err(mut idx) => {
                    if let Some(e) = ctx.props().events.get(idx) {
                        if announcement_range.contains(&(e.start_unixtime)) { // Check if the next event starts in the range
                            show_mobile_announcement = false;
                        } else {
                            idx = idx.overflowing_sub(1).0;
                            while let Some(e) = ctx.props().events.get(idx) { // Check if a few previous events end in the range
                                if announcement_range.contains(&(e.end_unixtime)) {
                                    show_mobile_announcement = false;
                                    break;
                                }
                                if e.end_unixtime < announcement_start - 6*3600 { // Avoid backtracking too much, 6h is enough
                                    break;
                                }
                                idx = idx.overflowing_sub(1).0;
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
            let selected_event_other_day = matches!(self.popup, PopupState::Opened { week_day, .. } if week_day != d && !mobile);
            let mut events = Vec::new();

            // Iterate over events, starting from the first one that starts during the current day
            let day_start = current_day.and_hms(0,0,0).timestamp() as u64;
            let mut idx = match ctx.props().events.binary_search_by_key(&day_start, |e| e.start_unixtime) {
                Ok(idx) => idx,
                Err(idx) => idx,
            };
            while let Some(e) = ctx.props().events.get(idx) {
                if e.start_unixtime > day_start + 24*3600 {
                    break;
                }
                events.push(html!{
                    <EventComp
                        week_day={d}
                        event={e.clone()}
                        day_start={current_day.and_hms(0,0,0).timestamp() as u64}
                        agenda_link={ctx.link().clone()}
                        show_announcement={show_mobile_announcement}>
                    </EventComp>
                });
                idx += 1;
            }

            let mut day_style = String::new();
            let mut day_name_style = String::new();
            if mobile {
                day_style.push_str(&format!("position: absolute; left: {}%;", (current_day.num_days_from_ce()-730000) * 20));
            } else {
                if selected_event_other_day {
                    day_style.push_str("opacity: 0; pointer-events: none;");
                    day_name_style.push_str("opacity: 0;");
                }
                if let PopupState::Opened { week_day, .. } = self.popup {
                    day_name_style.push_str(&format!("transform: translateX(calc(-100%*{week_day} + -10px*{week_day}))"));
                }
            }

            day_names.push(html! {
                <span id={if current_day == self.selected_day {"selected-day"} else {""}} style={day_name_style}>
                    { format_day(current_day.weekday(), current_day.day()) }
                </span>
            });
            days.push(html! {
                <div class="day" id={format!("day{d}")} style={day_style}>
                    { events }
                </div>
            });

            current_day = current_day.succ();
        }

        let calendar = html! {
            <Calendar
                agenda_link={ctx.link().clone()}
                day={self.selected_day.day()}
                month={self.selected_day.month()}
                year={self.selected_day.year()} />
        };
        let opt_popup = self.popup.as_option().map(|(week_day, event, _)|
            html! {
                <Popup
                    week_day = {week_day}
                    event = {event}
                    agenda_link = {ctx.link().clone()} />
            }
        );
        let popup_container_style = match &self.popup {
            PopupState::Opened { popup_size, .. } => match mobile {
                true => {
                    let body_height = window().doc().body().unwrap().client_height() as usize;
                    format!("top: -{body_height}px; height: {body_height}px;")
                }
                false => match popup_size {
                    Some(popup_size) => format!("left: -{popup_size}px; width: {popup_size}px;"),
                    None => "left: -70vw; width: 70vw;".to_string(),
                }
            },
            PopupState::Closing { popup_size, .. } => match mobile {
                true => {
                    let screen_height = window().inner_height().unwrap().as_f64().unwrap() as usize;
                    format!("height: {screen_height}px;")
                }
                false => match popup_size {
                    Some(popup_size) => format!("width: {popup_size}px;"),
                    None => "width: 70vw;".to_string(),
                }
            },
            PopupState::Closed => String::new(),
        };
        
        let day_container_style = if mobile {
            format!("right: {}%", 100 * (self.selected_day.num_days_from_ce() - 730000))
        } else if let PopupState::Opened { week_day, .. } = self.popup {
            let c = self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("right: calc((100%/6)*{week_day}); c: {};", c) // c is a workarround for a bug in Yew
        } else {
            String::new()
        };
        template_html!(
            "src/agenda/agenda.html",
            onclick_settings = {ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Settings))},
            onclick_previous = {ctx.link().callback(|_| AgendaMsg::Previous)},
            onclick_next = {ctx.link().callback(|_| AgendaMsg::Next)},
            opt_announcement = announcement,
            ...
        )
    }
}