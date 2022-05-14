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
mod change_password;
mod prelude;
mod translation;

use crate::{prelude::*, settings::SettingsPage, change_password::ChangePasswordPage};

#[derive(PartialEq)]
pub enum Page {
    Settings,
    ChangePassword,
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
}

#[derive(Serialize, Deserialize)]
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
                Some("settings") | Some("parametres") => link2.send_message(Msg::SilentSetPage(Page::Settings)),
                Some("agenda") => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                Some("change-password") | Some("changer-mot-de-passe")  => link2.send_message(Msg::SilentSetPage(Page::ChangePassword)),
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
            let link2 = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::load_events().await {
                    Ok(events) => link2.send_message(Msg::ScheduleSuccess(events)),
                    Err(e) => link2.send_message(Msg::ScheduleFailure(e)),
                }
            });
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
            Ok(hash) if hash == "#parametres" || hash == format!("#{}", te("parametres")) => Page::Settings,
            Ok(hash) if hash == "#changer-mot-de-passe" || hash == format!("#{}", te("changer-mot-de-passe")) => Page::ChangePassword,
            Ok(hash) if hash.is_empty() => Page::Agenda,
            Ok(hash) => {
                alert(format!("Page {hash} {}", t("introuvable")));
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
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ScheduleSuccess(events) => {
                self.events = events;
                true
            }
            Msg::UserInfoSuccess(user_info) => {
                self.user_info = Rc::new(Some(user_info));
                false
            }
            Msg::ScheduleFailure(api_error) => {
                alert(format!("{} {}", t("Impossible de charger les cours : "), api_error));
                false
            },
            Msg::UserInfoFailure(api_error) => {
                alert(format!("{} {}", t("Impossible de charger les informations utilisateur :") ,api_error));
                false
            },
            Msg::SetPage(page) => {
                let history = window().history().expect(t("Impossible de charger l'historique"));                
                match &page {
                    Page::Settings => history.push_state_with_url(&JsValue::from_str(t("parametres")), t("Paramètres"), Some(&format!("#{}", t("parametres")))).unwrap(),
                    Page::Agenda => history.push_state_with_url(&JsValue::from_str("agenda"), "Agenda", Some("/agenda")).unwrap(),
                    Page::ChangePassword => history.push_state_with_url(&JsValue::from_str(t("changer-mot-de-passe")), t("Changer son mot de passse"), Some(&format!("#{}", t("changer-mot-de-passe")))).unwrap(),
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
            Msg::Refresh => true,
        }
    }
    
    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        crate::colors::COLORS_CHANGED.store(false, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.page {
            Page::Agenda => self.view_agenda(ctx),
            Page::Settings => html!( <SettingsPage app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
            Page::ChangePassword => html!( <ChangePasswordPage app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
        }
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
    let document = window.document().unwrap();
    let element = document.get_element_by_id("render").unwrap();
    yew::start_app_in_element::<App>(element);
}
