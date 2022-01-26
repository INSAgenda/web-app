use agenda_parser::Event;
use chrono::{Datelike, Date, TimeZone, Weekday};
use event::EventGlobalData;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use yew::prelude::*;
use std::{rc::Rc, cell::RefCell};
use chrono_tz::Europe::Paris;

mod event;
mod settings;
mod agenda;
mod glider_selector;
mod util;
mod calendar;
mod slider;
mod api;
mod crash_handler;
use api::*;
pub use util::sleep;
use crate::settings::Settings;

pub enum Page {
    Settings,
    Agenda,
}

pub enum Msg {
    FetchSuccess(Vec<Event>),
    FetchFailure(ApiError),
    Previous,
    SwipePrevious,
    Next,
    SwipeNext,
    Goto {day: u32, month: u32, year: i32},
    SetPage(Page),
    SilentSetPage(Page),
}

pub struct App {
    selected_day: Date<chrono_tz::Tz>,
    event_global: Rc<EventGlobalData>,
    api_key: u64,
    counter: Rc<std::sync::atomic::AtomicU64>,
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

        let mut day_start = (now.timestamp() - (now.timestamp() + 1 * 3600) % 86400) as u64;
        match now.weekday().number_from_monday() {
            6 => day_start += 86400,
            7 => day_start += 2 * 86400,
            _ => (),
        };

        // Extract api key data
        let window = web_sys::window().unwrap();
        let local_storage = window.local_storage().unwrap().unwrap();
        let api_key = local_storage.get("api_key").map(|v| v.map(|v| v.parse()));
        let counter = local_storage.get("counter").map(|v| v.map(|v| v.parse()));
        let (api_key, counter) = match (api_key, counter) {
            (Ok(Some(Ok(api_key))), Ok(Some(Ok(counter)))) => (api_key, counter),
            _ => {
                window.location().replace("/login").unwrap();
                std::process::exit(0);
            },
        };
        let counter = Rc::new(std::sync::atomic::AtomicU64::new(counter));

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
            let counter2 = Rc::clone(&counter);
            let link2 = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::load_events(api_key, counter2).await {
                    Ok(events) => link2.send_message(Msg::FetchSuccess(events)),
                    Err(e) => link2.send_message(Msg::FetchFailure(e)),
                }
            });
        }

        Self {
            selected_day: now.date(),
            api_key,
            counter,
            events,
            page: Page::Agenda,
            slider: slider::SliderManager::init(ctx.link().clone()),
            event_global: Rc::new(EventGlobalData::default()),
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
                let history = web_sys::window().unwrap().history().unwrap();                
                match &page {
                    Page::Settings => history.push_state_with_url(&JsValue::from_str("settings"), "Settings", Some("#setttings")).unwrap(),
                    Page::Agenda => history.push_state_with_url(&JsValue::from_str("agenda"), "Agenda", Some("/agenda")).unwrap(),
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
                    true
                } else {
                    self.slider.borrow_mut().swipe_left(); // This function will send the SwipePrevious message
                    false
                }
            },
            Msg::SwipePrevious => {
                self.selected_day = self.selected_day.pred();
                false
            },
            Msg::Next => {
                if self.selected_day.weekday() ==  Weekday::Fri {
                    self.selected_day = self.selected_day.succ().succ().succ();
                    true
                } else {
                    self.slider.borrow_mut().swipe_right(); // This function will send the SwipeNext message
                    false
                }
            },
            Msg::SwipeNext => {
                self.selected_day = self.selected_day.succ();
                false
            },
            Msg::Goto {day, month, year} => {
                self.selected_day = Paris.ymd(year, month, day);
                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        self.slider.borrow_mut().set_selected_index(self.selected_day.weekday().num_days_from_monday());
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.page {
            Page::Agenda => self.view_agenda(ctx),
            Page::Settings => html!( <Settings app_link={ ctx.link().clone() } /> ),
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
