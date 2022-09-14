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

use crate::{prelude::*, settings::SettingsPage, change_data::ChangeDataPage};

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
    announcements: Vec<AnnouncementDesc>,
    displayed_announcement: Option<AnnouncementDesc>,
    page: Page,
    user_info: Rc<Option<UserInfo>>,
    slider: Rc<RefCell<slider::SliderManager>>,
}

fn refresh_events(app_link: Scope<App>) {
    wasm_bindgen_futures::spawn_local(async move {
        match api::load_events().await {
            Ok(events) => app_link.send_message(Msg::ScheduleSuccess(events)),
            Err(e) => app_link.send_message(Msg::ScheduleFailure(e)),
        }
    });
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

        // Update user info
        let mut skip_user_info_loading = false;
        let mut user_info = None;
        if let Some((last_updated, cached_user_info)) = api::load_cached_user_info() {
            if last_updated > now.timestamp() - 60*5 {
                skip_user_info_loading = true;
            }
            user_info = Some(cached_user_info);
        }
        if !skip_user_info_loading {
            let link2 = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::load_user_info().await {
                    Ok(events) => link2.send_message(Msg::UserInfoSuccess(events)),
                    Err(e) => link2.send_message(Msg::ApiFailure(e)),
                }
            });
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
                    Ok(events) => link2.send_message(Msg::AnnouncementsSuccess(events)),
                    Err(e) => e.handle_api_error(),
                }
            });
        }

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
        let unload = Closure::wrap(Box::new(move |event: web_sys::Event| {
            link.send_message(AppMsg::PushColors());

        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("unload", unload.as_ref().unchecked_ref()).unwrap();
        unload.forget();

        // Get colors
        crate::COLORS.as_ref().fetch_colors(ctx);

        // Auto-push colors every 30s if needed
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
                sleep(Duration::from_millis(500)).await;
                link2.send_message(Msg::Next);
                if weekday == Weekday::Sat {
                    sleep(Duration::from_millis(300)).await;
                    link2.send_message(Msg::Next);
                }
            });
        }

        Self {
            selected_day: now.date(),
            events,
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

                // If user's group changed, update the events
                if let Some(old_user_info) = self.user_info.as_ref() {
                    if old_user_info.group_desc != user_info.group_desc {
                        self.events.clear();
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
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));
                true
            },
            Msg::Next => {
                if self.selected_day.weekday() ==  Weekday::Fri {
                    self.selected_day = self.selected_day.succ().succ().succ();
                } else {
                    self.selected_day = self.selected_day.succ();
                }
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));
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
                let colors = crate::COLORS.as_ref();
                colors.update_colors(new_colors);
                true
            },
            Msg::PushColors() => {
                let colors = crate::COLORS.as_ref();
                colors.push_colors();
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
            Page::ChangePassword => html!( <ChangeDataPage kind="new_password" app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
            Page::ChangeEmail => html!( <ChangeDataPage kind="email" app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
            Page::ChangeGroup => html!( <ChangeDataPage kind="group" app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
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
