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

use crate::{prelude::*, settings::SettingsPage, change_password::ChangePasswordPage};

#[derive(PartialEq)]
pub enum Page {
    Settings,
    ChangePassword,
    Agenda,
}

pub enum Msg {
    FetchSuccess(Vec<Event>),
    FetchFailure(ApiError),
    Previous,
    Next,
    Goto {day: u32, month: u32, year: i32},
    SetPage(Page),
    SilentSetPage(Page),
    Refresh,
}

pub struct App {
    selected_day: Date<chrono_tz::Tz>,
    events: Vec<Event>,
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
        let window = window();

        let link2 = ctx.link().clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::PopStateEvent| {
            let state = e.state().as_string();
            match state.as_deref() {
                Some("settings") => link2.send_message(Msg::SilentSetPage(Page::Settings)),
                Some("agenda") => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                _ if e.state().is_null() => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                _ => log!("Unknown pop state: {:?}", e.state()),
            }
        }) as Box<dyn FnMut(_)>);
        window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();

        // Update events
        let mut skip_event_loading = false;
        let mut events = Vec::new();
        if let Some((last_updated, cached_events)) = api::load_cache() {
            if last_updated > now.timestamp() - 3600*5 && !cached_events.is_empty() {
                skip_event_loading = true;
            }
            events = cached_events;
        }
        if !skip_event_loading {
            let link2 = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::load_events().await {
                    Ok(events) => link2.send_message(Msg::FetchSuccess(events)),
                    Err(e) => link2.send_message(Msg::FetchFailure(e)),
                }
            });
        }

        Self {
            selected_day: now.date(),
            events,
            page: Page::Agenda,
            slider: slider::SliderManager::init(ctx.link().clone(), -20 * (now.date().num_days_from_ce() - 730000)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchSuccess(events) => {
                self.events = events;
                true
            }
            Msg::FetchFailure(_) => {
                log!("Todo");
                true
            },
            Msg::SetPage(page) => {
                let history = window().history().unwrap();                
                match &page {
                    Page::Settings => history.push_state_with_url(&JsValue::from_str("settings"), "Settings", Some("#setttings")).unwrap(),
                    Page::Agenda => history.push_state_with_url(&JsValue::from_str("agenda"), "Agenda", Some("/agenda")).unwrap(),
                    Page::ChangePassword => history.push_state_with_url(&JsValue::from_str("change_password"), "Change password", Some("#change_password")).unwrap(),
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
            Page::Settings => html!( <SettingsPage app_link={ ctx.link().clone() } /> ),
            Page::ChangePassword => html!( <ChangePasswordPage app_link={ ctx.link().clone() } /> ),
        }
    }
}

fn main() {
    let window = web_sys::window().expect("Please run the program in a browser context");
    let document = window.document().unwrap();
    let element = document.get_element_by_id("render").unwrap();
    yew::start_app_in_element::<App>(element);
}
