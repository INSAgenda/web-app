mod alert;
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

#[derive(PartialEq)]
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
    ScheduleFailure(ApiError),
    UserInfoFailure(ApiError),
    Previous,
    Next,
    Goto {day: u32, month: u32, year: i32},
    SetPage(Page),
    SilentSetPage(Page),
    Refresh,
    SetSliderState(bool),
}

#[derive(PartialEq)]
enum InputAction{
    None,
    Next,
    Previous,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo {
    /// The count of api keys
    pub api_key_count: u64,
    /// Last password modification timestamp.
    /// Can be `None` if the user has no password or if the user has never changed his password since the addition of the tracking feature.
    pub last_password_mod: Option<i64>,
    /// The email associated with its verification state
    pub email: (String, bool),
    pub group_desc: GroupDescriptor,
}

pub struct App {
    selected_day: Date<chrono_tz::Tz>,
    events: Vec<RawEvent>,
    page: Page,
    user_info: Rc<Option<UserInfo>>,
    slider: Rc<RefCell<slider::SliderManager>>,

    /// input tracking
    n_reapeted_input: u16,
    sum_time_beetween: i64,
    last_input_time: i64,
    last_action: InputAction,
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
                    Err(e) => link2.send_message(Msg::UserInfoFailure(e)),
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

        Self {
            selected_day: now.date(),
            events,
            page,
            user_info: Rc::new(user_info),
            slider: slider::SliderManager::init(ctx.link().clone(), -20 * (now.date().num_days_from_ce() - 730000)),
            n_reapeted_input: 0,
            sum_time_beetween: 0,
            last_input_time: 0,
            last_action: InputAction::None,
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
            Msg::ScheduleFailure(api_error) => {
                match api_error {
                    ApiError::Known(error) if error.kind == "counter_too_low" => {
                        log!("Counter too low");
                        counter_to_the_moon();
                        refresh_events(ctx.link().clone());
                    }
                    ApiError::Known(error) => alert(format!("Failed to load events: {}", error)),
                    ApiError::Unknown(error) => {
                        log!("Failed to load events: {:?}", error);
                        alert("Failed to load events.");
                    }
                }
                false
            },
            Msg::UserInfoFailure(api_error) => {
                match api_error {
                    ApiError::Known(error) if error.kind == "counter_too_low" => {
                        log!("Counter too low");
                        counter_to_the_moon();
                    }
                    ApiError::Known(error) => alert(format!("Failed to load user info: {}", error)),
                    ApiError::Unknown(error) => {
                        log!("Failed to load user info: {:?}", error);
                        alert("Failed to load user info.");
                    }
                }
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
                self.handle_action(ctx, InputAction::Previous);
                true
            },
            Msg::Next => {
                if self.selected_day.weekday() ==  Weekday::Fri {
                    self.selected_day = self.selected_day.succ().succ().succ();
                } else {
                    self.selected_day = self.selected_day.succ();
                }
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));
                self.handle_action(ctx, InputAction::Next);
                true
            },
            Msg::Goto {day, month, year} => {
                self.selected_day = Paris.ymd(year, month, day);
                self.slider.borrow_mut().set_offset(-20 * (self.selected_day.num_days_from_ce() - 730000));

                true
            }
            Msg::Refresh => true,
            Msg::SetSliderState(state) => {
                let mut slider = self.slider.borrow_mut();
                match state {
                    true => slider.enable(),
                    false => slider.disable(),
                }
                true
            }
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

impl App{
    fn handle_action(&mut self, ctx: &Context<Self>, action: InputAction) {
        let now = chrono::Local::now().timestamp();
        let time_between = now - self.last_input_time;

        if self.last_action == action && time_between < 3 {
            self.n_reapeted_input += 1;
            self.sum_time_beetween += time_between;
            // check if is equal to 4 and not if is upper than 3 because if there is no other courses after it will test all courses each click
            if self.n_reapeted_input == 3 {
                let mut travel_direction = None;
                match action {
                    InputAction::Previous => {
                        travel_direction = Some(true);
                        log!("Previous");
                    }
                    InputAction::Next => {
                        travel_direction = Some(false);
                        log!("Next");
                    }
                    _ => {}
                }
                if let Some(dir) = travel_direction{
                    let mut close_i = None;
                    let mut min_d = if dir {std::i64::MIN} else {std::i64::MAX};
                    for (i, event) in self.events.iter().enumerate(){
                        let d = (event.start_unixtime as i64) - self.selected_day.and_hms(0,0,0).timestamp();
                        if (dir && d < 0 && d > min_d) || (!dir && d > 0 && d < min_d){
                            close_i = Some(i);
                            min_d = d;
                        }
                    }
                    if let Some(i) = close_i{
                        let date = Paris.timestamp(self.events[i].start_unixtime as i64, 0).date();
                        ctx.link().send_message(AppMsg::Goto { day: date.day(), month: date.month(), year: date.year() }); 
                        }
                }

            }
        } else {
            self.n_reapeted_input = 0;
            self.sum_time_beetween = 0;
        }

        self.last_action = action;
        self.last_input_time = now;   
    }
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
