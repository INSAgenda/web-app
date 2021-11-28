use agenda_parser::Event;
use chrono::{offset::FixedOffset, Weekday, Datelike, TimeZone, Timelike};
use event::EventGlobalData;
use wasm_bindgen::{JsCast, JsValue};
use yew::{
    prelude::*,
    format::Nothing,
    services::fetch::{FetchService, FetchTask, Request, Response},
};

mod event;
mod settings;
mod agenda;
use crate::{event::EventComp, settings::Settings};

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

enum Page {
    Settings,
    Agenda,
}

enum Msg {
    FetchSuccess(Vec<Event>),
    FetchFailure(anyhow::Error),
    PreviousWeek,
    NextWeek,
    OpenSettings,
}

struct App {
    weekstart: u64,
    event_global: std::rc::Rc<EventGlobalData>,
    api_key: u64,
    counter: u64,
    events: Vec<Event>,
    page: Page,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let date = chrono::Local::now();
        let date = date.with_timezone(&chrono::offset::FixedOffset::east(1 * 3600));

        let daystart = (date.timestamp() - (date.timestamp() + 1 * 3600) % 86400) as u64;
        let mut weekstart = daystart - (date.weekday().number_from_monday() as u64 - 1) * 86400;
        if date.weekday().number_from_monday() >= 6 {
            weekstart += 7 * 86400;
        }

        let window = web_sys::window().unwrap();
        let local_storage = window.local_storage().unwrap().unwrap();
        let api_key = local_storage.get("api_key").unwrap().expect("missing api key").parse().unwrap();
        let counter = local_storage.get("counter").unwrap().expect("missing counter").parse().unwrap();

        let mut app = Self {
            weekstart,
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
            Msg::OpenSettings => {
                self.page = Page::Settings;
                true
            },
            Msg::PreviousWeek => {
                self.weekstart -= 7 * 86400;
                true
            }
            Msg::NextWeek => {
                self.weekstart += 7 * 86400;
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
            Page::Settings => html!( <Settings /> ),
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
