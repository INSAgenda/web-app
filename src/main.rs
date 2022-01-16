use agenda_parser::Event;
use chrono::{Datelike, TimeZone, FixedOffset, NaiveTime};
use event::EventGlobalData;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use yew::prelude::*;
use std::{rc::Rc, cell::RefCell};

mod event;
mod settings;
mod agenda;
mod glider_selector;
mod util;
mod calendar;
mod slider;
mod api;
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
    Next,
    Goto {day: u32, month: u32, year: i32},
    SetPage(Page),
    SilentSetPage(Page),
}

pub struct App {
    day_start: u64,
    event_global: Rc<EventGlobalData>,
    api_key: u64,
    counter: Rc<std::sync::atomic::AtomicU64>,
    events: Vec<Event>,
    page: Page,
    slider_manager: Rc<RefCell<slider::SliderManager>>,
    link: Rc<ComponentLink<Self>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link = Rc::new(link);

        let date = chrono::Local::now();
        let date = date.with_timezone(&FixedOffset::east(1 * 3600));

        let mut day_start = (date.timestamp() - (date.timestamp() + 1 * 3600) % 86400) as u64;
        match date.weekday().number_from_monday() {
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

        let link2 = Rc::clone(&link);
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
        let counter2 = Rc::clone(&counter);
        let link2 = Rc::clone(&link);
        wasm_bindgen_futures::spawn_local(async move {
            match api::load_events(api_key, counter2).await {
                Ok(events) => link2.send_message(Msg::FetchSuccess(events)),
                Err(e) => link2.send_message(Msg::FetchFailure(e)),
            }
        });

        Self {
            day_start,
            api_key,
            counter,
            events: Vec::new(),
            page: Page::Agenda,
            slider_manager: slider::SliderManager::init(),
            event_global: Rc::new(EventGlobalData::default()),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
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
                    Page::Agenda => history.push_state_with_url(&JsValue::from_str("agenda"), "Agenda", Some("/agenda/index.html")).unwrap(),
                }
                self.page = page;
                true
            },
            Msg::SilentSetPage(page) => {
                self.page = page;
                true
            },
            Msg::Previous => {
                self.day_start -= 86400;
                true
            }
            Msg::Next => {
                self.day_start += 86400;
                true
            }
            Msg::Goto {day, month, year} => {
                let datetime = FixedOffset::east(1 * 3600).ymd(year, month, day).and_time(NaiveTime::from_hms(0, 0, 0)).unwrap();
                self.day_start = datetime.timestamp() as u64;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match &self.page {
            Page::Agenda => self.view_agenda(),
            Page::Settings => html!( <Settings app_link=Rc::clone(&self.link) /> ),
        }
    }
}

impl App {
    fn week_start(&self) -> u64 {
        let datetime = chrono::offset::FixedOffset::east(1 * 3600).timestamp(self.day_start as i64, 0);
        self.day_start - (datetime.weekday().number_from_monday() as u64 - 1) * 86400
    }
}

fn main() {
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}
