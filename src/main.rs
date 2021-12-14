use agenda_parser::Event;
use chrono::{Datelike, TimeZone};
use event::EventGlobalData;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use yew::{
    prelude::*,
    format::Nothing,
    services::fetch::{FetchService, FetchTask, Request, Response},
};
use std::rc::Rc;

mod event;
mod settings;
mod agenda;
mod glider_selector;
mod util;
mod calendar;
pub use util::sleep;
use crate::settings::Settings;

pub enum Page {
    Settings,
    Agenda,
}

pub enum Msg {
    FetchSuccess(Vec<Event>),
    FetchFailure(anyhow::Error),
    Previous,
    Next,
    SetPage(Page),
    SilentSetPage(Page),
}

pub struct App {
    day_start: u64,
    event_global: Rc<EventGlobalData>,
    api_key: u64,
    counter: u64,
    events: Vec<Event>,
    page: Page,
    fetch_task: Option<FetchTask>,
    link: Rc<ComponentLink<Self>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link = Rc::new(link);

        let date = chrono::Local::now();
        let date = date.with_timezone(&chrono::offset::FixedOffset::east(1 * 3600));

        let mut day_start = (date.timestamp() - (date.timestamp() + 1 * 3600) % 86400) as u64;
        match date.weekday().number_from_monday() {
            6 => day_start += 86400,
            7 => day_start += 2 * 86400,
            _ => (),
        };

        let window = web_sys::window().unwrap();
        let local_storage = window.local_storage().unwrap().unwrap();
        let api_key = local_storage.get("api_key").unwrap().expect("missing api key").parse().unwrap();
        let counter = local_storage.get("counter").unwrap().expect("missing counter").parse().unwrap();

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

        let mut app = Self {
            day_start,
            api_key,
            counter,
            fetch_task: None,
            events: Vec::new(),
            page: Page::Agenda,
            event_global: std::rc::Rc::new(EventGlobalData::default()),
            link,
        };
        app.new_fetch_task(0..i64::MAX);

        app
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

pub fn gen_code(api_key: u64, counter: u64) -> u64 {
    let mut key = (api_key + 143 * counter) as u128;
    for _ in 0..11 {
        key = key * key + 453;
        if key <= 0xffff_ffff {
            key += 0x4242424242424242424242424242;
        }
        key &= 0x0000_0000_ffff_ffff_ffff_ffff_0000_0000;
        key >>= 32;
    }
    key as u64
}

impl App {
    fn week_start(&self) -> u64 {
        let datetime = chrono::offset::FixedOffset::east(1 * 3600).timestamp(self.day_start as i64, 0);
        let week_start = self.day_start - (datetime.weekday().number_from_monday() as u64 - 1) * 86400;
        
        week_start
    }

    fn new_fetch_task(&mut self, time_range: std::ops::Range<i64>) {
        let request = Request::get(format!("http://127.0.0.1:8080/api/schedule/?start_timestamp=0&end_timestamp={}", u64::MAX))
            .header("Api-Key", format!("{}-{}-{}", self.api_key, self.counter, gen_code(self.api_key, self.counter)))
            .body(Nothing)
            .expect("Could not build request.");
        self.counter += 1;
        let window = web_sys::window().unwrap();
        let local_storage = window.local_storage().unwrap().unwrap();
        local_storage.set("counter", &self.counter.to_string()).unwrap();

        let callback = self
            .link
            .callback(|response: Response<Result<String, anyhow::Error>>| {
                if response.status() != 200 {
                    return Msg::FetchFailure(anyhow::Error::msg(format!(
                        "Failed request. {:?}",
                        response.into_body()
                    )));
                }

                let body = match response.into_body() {
                    Ok(body) => body,
                    Err(e) => {
                        return Msg::FetchFailure(anyhow::Error::msg(format!(
                            "Cannot read response body. {:?}",
                            e
                        )));
                    }
                };

                match serde_json::from_str(&body) {
                    Ok(results) => Msg::FetchSuccess(results),
                    Err(e) => Msg::FetchFailure(anyhow::Error::msg(format!(
                        "Cannot deserialize response. {:?}",
                        e
                    ))),
                }
            });

        let task = FetchService::fetch(request, callback).expect("failed to start request");
        self.fetch_task = Some(task);
    }
}

fn main() {
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}
