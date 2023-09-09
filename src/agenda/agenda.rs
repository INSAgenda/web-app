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
    counter: AtomicUsize,
}

pub enum AgendaMsg {
    Previous,
    Next,
    Goto{ day: u32, month: u32, year: i32 },
    Refresh,
    PushColors,
    AppMsg(AppMsg),
}

#[derive(Properties, Clone)]
pub struct AgendaProps {
    pub app_link: AppLink,
    pub events: Rc<Vec<RawEvent>>,
    pub popup: Option<(RawEvent, bool, Option<usize>)>,
    pub profile_src: Option<String>,
    pub user_info: Rc<Option<UserInfo>>,
    pub comment_counts: Rc<CommentCounts>,
    pub seen_comment_counts: Rc<CommentCounts>,
    pub friends: Rc<Option<FriendLists>>,
}

impl PartialEq for AgendaProps {
    fn eq(&self, other: &Self) -> bool {
        !COLORS_CHANGED.load(Ordering::Relaxed)
            && self.events == other.events
            && self.popup == other.popup
            && self.user_info == other.user_info
            && self.comment_counts == other.comment_counts
            && self.seen_comment_counts == other.seen_comment_counts
    }
}

impl Component for Agenda {
    type Message = AgendaMsg;
    type Properties = AgendaProps;

    fn create(ctx: &Context<Self>) -> Self {
        let now = chrono::Local::now();
        let now = now.with_timezone(&Paris);

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
        let has_event = has_event_on_day(&ctx.props().events, now.date_naive(), Weekday::Sat);
        if now.hour() >= 19 || weekday == Weekday::Sun || (weekday == Weekday::Sat && !has_event) {
            let link2 = ctx.link().clone();
            spawn_local(async move {
                sleep(Duration::from_millis(500)).await;
                link2.send_message(AgendaMsg::Next);
            });
        }
        
        // Disable slider if popup is open
        let slider = slider::SliderManager::init(ctx.link().clone(), -20 * (now.date_naive().num_days_from_ce() - 730000));
        if ctx.props().popup.is_some() {
            slider.borrow_mut().disable();
        }

        Self {
            selected_day: now.date_naive(),
            slider,
            counter: AtomicUsize::new(0),
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
            },
            AgendaMsg::PushColors => {
                crate::COLORS.push_colors();
                false
            },
            AgendaMsg::AppMsg(msg) => {
                ctx.props().app_link.send_message(msg);
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if ctx.props().popup.is_some() {
            self.slider.borrow_mut().disable();
        } else {
            self.slider.borrow_mut().enable();
        }
        true
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
            let selected_event_other_day = !mobile && ctx.props().popup.as_ref().map(|(e,is_closing,_)| !is_closing && !(day_start..day_start+86400).contains(&e.start_unixtime)).unwrap_or(false);
            let mut events = Vec::new();

            // Iterate over events, starting from the first one that starts during the current day
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
                        day_start={day_start}
                        agenda_link={ctx.link().clone()}
                        comment_counts={Rc::clone(&ctx.props().comment_counts)}
                        seen_comment_counts={Rc::clone(&ctx.props().seen_comment_counts)}>
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
                if let Some((event, false, _)) = &ctx.props().popup {
                    let week_day = Paris.from_local_datetime(&NaiveDateTime::from_timestamp_opt(event.start_unixtime as i64, 0).unwrap()).unwrap().weekday().num_days_from_monday();
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

            current_day += chrono::Duration::days(1);
        }

        let calendar = html! {
            <Calendar
                agenda_link={ctx.link().clone()}
                day={self.selected_day.day()}
                month={self.selected_day.month()}
                year={self.selected_day.year()} />
        };
        let opt_popup = ctx.props().popup.as_ref().map(|(event, _, _)|
            html! {
                <Popup
                    event={event.clone()}
                    agenda_link={ctx.link().clone()}
                    friends={Rc::clone(&ctx.props().friends)}
                    user_info={Rc::clone(&ctx.props().user_info)} />
            }
        );
        let popup_container_style = match &ctx.props().popup {
            Some((_, false, popup_size)) => match mobile {
                true => {
                    let body_height = window().doc().body().unwrap().client_height() as usize;
                    format!("top: calc(-{body_height}px + 4rem); height: calc({body_height}px - 4rem);") // 4rem is the height of the tabbar
                }
                false => match popup_size {
                    Some(popup_size) => format!("left: calc(-{popup_size}px - 4rem); width: calc({popup_size}px + 4rem);"),
                    None => "left: -70vw; width: 70vw;".to_string(),
                }
            },
            Some((_, true, popup_size)) => match mobile {
                true => {
                    let screen_height = window().inner_height().unwrap().as_f64().unwrap() as usize;
                    format!("height: {screen_height}px;")
                }
                false => match popup_size {
                    Some(popup_size) => format!("width: {popup_size}px;"),
                    None => "width: 70vw;".to_string(),
                }
            },
            None => String::new(),
        };
        
        let day_container_style = if mobile {
            format!("right: {}%", 100 * (self.selected_day.num_days_from_ce() - 730000))
        } else if let Some((event, false, _)) = &ctx.props().popup {
            let c = self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let week_day = Paris.from_local_datetime(&NaiveDateTime::from_timestamp_opt(event.start_unixtime as i64, 0).unwrap()).unwrap().weekday().num_days_from_monday();
            format!("right: calc((100%/6)*{week_day}); c: {};", c) // c is a workarround for a bug in Yew
        } else {
            String::new()
        };
        template_html!(
            "src/agenda/agenda.html",
            onclick_rick = {ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Rick))},
            onclick_previous = {ctx.link().callback(|_| AgendaMsg::Previous)},
            onclick_next = {ctx.link().callback(|_| AgendaMsg::Next)},
            ...
        )
    }
}
