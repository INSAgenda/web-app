mod alert;
mod announcements;
mod event;
mod settings;
mod agenda;
mod glider_selector;
mod util;
mod calendar;
mod slider;
mod api;
mod crash_handler;
mod colors;
mod change_data;
mod prelude;
mod translation;

use crate::{prelude::*, settings::SettingsPage, change_data::ChangeDataPage, slider::width};

pub enum Page {
    Settings,
    ChangePassword,
    ChangeEmail,
    ChangeGroup,
    Agenda,
}

pub enum Msg {
    ScheduleSuccess(Vec<RawEvent>),
    UserInfoSuccess(UserInfo),
    AnnouncementsSuccess(Vec<AnnouncementDesc>),
    GroupsSuccess(Vec<GroupDesc>),
    ScheduleFailure(ApiError),
    ApiFailure(ApiError),
    FetchColors(HashMap<String, String>),
    PushColors(),
    Previous,
    Next,
    Goto {day: u32, month: u32, year: i32},
    SetPage(Page),
    SilentSetPage(Page),
    Refresh,
    SetSliderState(bool),
    CloseAnnouncement,
}

pub struct App {
    selected_day: Date<chrono_tz::Tz>,
    events: Vec<RawEvent>,
    groups: Rc<Vec<GroupDesc>>,
    user_info: Rc<Option<UserInfo>>,
    announcements: Vec<AnnouncementDesc>,
    displayed_announcement: Option<AnnouncementDesc>,
    page: Page,
    slider: Rc<RefCell<slider::SliderManager>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        crash_handler::init();

        let now = chrono::Local::now();
        let now = now.with_timezone(&Paris);

        let link2 = ctx.link().clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::PopStateEvent| {
            let state = e.state().as_string();
            match state.as_deref() {
                Some("settings") => link2.send_message(Msg::SilentSetPage(Page::Settings)),
                Some("agenda") => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                Some("change-password") => link2.send_message(Msg::SilentSetPage(Page::ChangePassword)),
                Some("change-email") => link2.send_message(Msg::SilentSetPage(Page::ChangeEmail)),
                Some("change-group") => link2.send_message(Msg::SilentSetPage(Page::ChangeGroup)),
                _ if e.state().is_null() => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                _ => alert(format!("Unknown pop state: {:?}", e.state())),
            }
        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();

        // Update data
        let events = init_events(now, ctx.link().clone());
        let user_info = init_user_info(now, ctx.link().clone());
        let groups = init_groups(now, ctx.link().clone());
        let announcements = init_announcements(now, ctx.link().clone());
        let displayed_announcement = select_announcement(&announcements, &user_info);

        // Detect page
        let page = match window().location().hash() {
            Ok(hash) if hash == "#settings" => Page::Settings,
            Ok(hash) if hash == "#change-password" => Page::ChangePassword,
            Ok(hash) if hash == "#change-email" => Page::ChangeEmail,
            Ok(hash) if hash == "#change-group" => Page::ChangeGroup,
            Ok(hash) if hash.is_empty() => Page::Agenda,
            Ok(hash) => {
                alert(format!("Page {hash} not found"));
                Page::Agenda
            },
            _ => Page::Agenda,
        };

        let link = ctx.link().clone();
        let unload = Closure::wrap(Box::new(move |_: web_sys::Event| {
            link.send_message(AppMsg::PushColors());
        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("unload", unload.as_ref().unchecked_ref()).unwrap();
        unload.forget();

        // Get colors
        crate::COLORS.fetch_colors(ctx);

        // Auto-push colors every 15s if needed
        let link = ctx.link().clone();
        let push_colors = Closure::wrap(Box::new(move || {
            link.send_message(AppMsg::PushColors());
        }) as Box<dyn FnMut()>);

        match window().set_interval_with_callback_and_timeout_and_arguments(
            push_colors.as_ref().unchecked_ref(),
            1000*15,
            &Array::new(),
        ) {
            Ok(_) => (),
            Err(e) => sentry_report(JsValue::from(&format!("Failed to set timeout: {:?}", e))),
        }
        push_colors.forget();

        // Switch to next day if it's late or to monday if it's weekend
        let weekday = now.weekday();
        if now.hour() >= 19 || weekday == Weekday::Sat || weekday == Weekday::Sun {
            let link2 = ctx.link().clone();
            spawn_local(async move {
                if width() <= 1000 {
                    sleep(Duration::from_millis(500)).await;
                }
                link2.send_message(Msg::Next);
                if weekday == Weekday::Sat {
                    if width() <= 1000 { 
                        sleep(Duration::from_millis(300)).await;
                    }
                    link2.send_message(Msg::Next);
                }
            });
        }

        Self {
            selected_day: now.date(),
            events,
            groups: Rc::new(groups),
            page,
            announcements,
            displayed_announcement,
            user_info: Rc::new(user_info),
            slider: slider::SliderManager::init(ctx.link().clone(), -20 * (now.date().num_days_from_ce() - 730000)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ScheduleSuccess(events) => {
                self.events = events;
                true
            }
            Msg::UserInfoSuccess(user_info) => {
                let mut should_refresh = false;

                // If user's group changed, update the events and the announcement
                if let Some(old_user_info) = self.user_info.as_ref() {
                    if old_user_info.user_groups != user_info.user_groups {
                        self.events.clear();
                        self.displayed_announcement = select_announcement(&self.announcements, &Some(user_info.clone()));
                        refresh_events(ctx.link().clone());
                        should_refresh = true;
                    }
                }

                // Update user info
                api::save_user_info_cache(&user_info);
                self.user_info = Rc::new(Some(user_info));

                should_refresh
            }
            Msg::AnnouncementsSuccess(announcements) => {
                self.announcements = announcements;
                false // Don't think we should refresh display of the page because it would cause high inconvenience and frustration to the users
            }
            Msg::GroupsSuccess(groups) => {
                self.groups = Rc::new(groups);
                false
            }
            Msg::ScheduleFailure(api_error) => {
                api_error.handle_api_error();
                match api_error {
                    ApiError::Known(error) if error.kind == "counter_too_low" => {
                        refresh_events(ctx.link().clone());
                    }
                    _ => {},
                }
                false
            },
            Msg::ApiFailure(api_error) => {
                api_error.handle_api_error();
                false
            },
            Msg::SetPage(page) => {
                let history = window().history().expect("Failed to access history");                
                match &page {
                    Page::Settings => history.push_state_with_url(&JsValue::from_str("settings"), "Settings", Some("#settings")).unwrap(),
                    Page::Agenda => history.push_state_with_url(&JsValue::from_str("agenda"), "Agenda", Some("/agenda")).unwrap(),
                    Page::ChangePassword => history.push_state_with_url(&JsValue::from_str("change-password"), "Change password", Some("#change-password")).unwrap(),
                    Page::ChangeEmail => history.push_state_with_url(&JsValue::from_str("change-email"), "Change email", Some("#change-email")).unwrap(),
                    Page::ChangeGroup => history.push_state_with_url(&JsValue::from_str("change-group"), "Change group", Some("#change-group")).unwrap(),
                }
                self.page = page;
                true
            },
            Msg::SilentSetPage(page) => {
                self.page = page;
                true
            },
            Msg::Previous => {
                if self.selected_day.weekday() == Weekday::Mon {
                    self.selected_day = self.selected_day.pred().pred().pred();
                } else {
                    self.selected_day = self.selected_day.pred();
                }
                let _ = self.slider.try_borrow_mut().map(|mut s| s.set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000)));
                true
            },
            Msg::Next => {
                if self.selected_day.weekday() ==  Weekday::Fri {
                    self.selected_day = self.selected_day.succ().succ().succ();
                } else {
                    self.selected_day = self.selected_day.succ();
                }
                let _ = self.slider.try_borrow_mut().map(|mut s| s.set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000)));
                true
            },
            Msg::Goto {day, month, year} => {
                self.selected_day = Paris.ymd(year, month, day);
                true
            }
            Msg::Refresh => {
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
            Msg::CloseAnnouncement => update_close_announcement(self),
            Msg::SetSliderState(state) => {
                let mut slider = self.slider.borrow_mut();
                match state {
                    true => slider.enable(),
                    false => slider.disable(),
                }
                true
            },
            Msg::FetchColors(new_colors) => {
                crate::COLORS.update_colors(new_colors);
                true
            },
            Msg::PushColors() => {
                crate::COLORS.push_colors();
                false
            },
  
        }
    }
    
    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        crate::colors::COLORS_CHANGED.store(false, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.page {
            Page::Agenda => self.view_agenda(ctx),
            Page::Settings => html!( <SettingsPage app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
            Page::ChangePassword => html!(
                <ChangeDataPage
                    kind="new_password"
                    app_link={ ctx.link().clone() }
                    user_info={Rc::clone(&self.user_info)}
                    groups={Rc::clone(&self.groups)} />
            ),
            Page::ChangeEmail => html!(
                <ChangeDataPage
                    kind="email"
                    app_link={ ctx.link().clone() }
                    user_info={Rc::clone(&self.user_info)}
                    groups={Rc::clone(&self.groups)} />
            ),
            Page::ChangeGroup => html!(
                <ChangeDataPage
                    kind="group"
                    app_link={ ctx.link().clone() }
                    user_info={Rc::clone(&self.user_info)}
                    groups={Rc::clone(&self.groups)} />
            ),
        }
    }
}

/// Redirect the user
fn redirect(page: &str){
    let _ = window().location().set_href(page);
}

/// Prevent webdrivers from accessing the page
fn stop_bots(window: &web_sys::Window) {
    if js_sys::Reflect::get(&window.navigator(), &"webdriver".to_string().into()).unwrap().as_bool().unwrap_or(false) {
        panic!("Your browser failed load this page");
    }
}

/// Install service worker for offline access
fn install_sw(window: &web_sys::Window) {
    let future = JsFuture::from(window.navigator().service_worker().register("/sw.js"));
    spawn_local(async move {
        match future.await {
            Ok(_) => log!("Service worker doing well"),
            Err(e) => alert(format!("Failed to register service worker: {:?}", e)),
        }
    })
}

fn main() {
    let window = web_sys::window().expect("Please run the program in a browser context");
    stop_bots(&window);
    install_sw(&window);
    let doc = window.doc();
    let element = doc.get_element_by_id("render").unwrap();
    yew::start_app_in_element::<App>(element);
}
