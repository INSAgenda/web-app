use agenda_parser::Event;
use chrono::{offset::FixedOffset, Weekday, Datelike, TimeZone, Timelike};
use wasm_bindgen::{JsCast, JsValue};
use yew::{
    prelude::*,
    format::Nothing,
    services::fetch::{FetchService, FetchTask, Request, Response},
};

#[allow(unused_macros)]
macro_rules! log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

enum Msg {
    FetchSuccess(Vec<Event>),
    FetchFailure(anyhow::Error),
    PreviousWeek,
    NextWeek,
}

struct App {
    weekstart: i64,
    api_key: u64,
    counter: u64,
    events: Vec<Event>,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let date = chrono::Local::now();
        let date = date.with_timezone(&chrono::offset::FixedOffset::east(1 * 3600));

        let daystart = date.timestamp() - (date.timestamp() + 1 * 3600) % 86400;
        let mut weekstart = daystart - (date.weekday().number_from_monday() as i64 - 1) * 86400;
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
        let mut days = Vec::new();
        let mut day_names = Vec::new();
        for offset in 0..5 {
            let datetime =
                FixedOffset::east(1 * 3600).timestamp(self.weekstart + offset * 86400, 0);
            let day = datetime.day();
            let month = match datetime.month() {
                1 => "Janvier",
                2 => "Février",
                3 => "Mars",
                4 => "Avril",
                5 => "Mai",
                6 => "Juin",
                7 => "Juillet",
                8 => "Août",
                9 => "Septembre",
                10 => "Octobre",
                11 => "Novembre",
                12 => "Décembre",
                _ => unreachable!(),
            };

            let dayname = match datetime.weekday() {
                Weekday::Mon => "Lundi",
                Weekday::Tue => "Mardi",
                Weekday::Wed => "Mercredi",
                Weekday::Thu => "Jeudi",
                Weekday::Fri => "Vendredi",
                Weekday::Sat => "Samedi",
                Weekday::Sun => "Dimanche",
            };

            let mut events = Vec::new();
            for event in &self.events {
                if (event.start_unixtime as i64) > datetime.timestamp()
                    && (event.start_unixtime as i64) < datetime.timestamp() + 86400
                {
                    let start_time = FixedOffset::east(1 * 3600).timestamp(event.start_unixtime as i64, 0);
                    let end_time = FixedOffset::east(1 * 3600).timestamp(event.end_unixtime as i64, 0);
                    let start_time_is_common = [(8,0), (9,30), (9,45), (11,15), (11,30), (13,0), (15,0), (16,30), (16,45), (18,15)]
                        .contains(&(start_time.hour(), start_time.minute()));
                    let end_time_is_common = [(8,0), (9,30), (9,45), (11,15), (11,30), (13,0), (15,0), (16,30), (16,45), (18,15)]
                        .contains(&(end_time.hour(), end_time.minute()));
                    let times_are_common = start_time_is_common && end_time_is_common;

                    let sec_offset = event.start_unixtime - (self.weekstart as u64 + offset as u64 * 86400 + 8*3600);
                    let px_offset = 106.95/(6300.0)*sec_offset as f64;
                    let px_height = 106.95/(6300.0)*(event.end_unixtime - event.start_unixtime) as f64;

                    let name = match &event.kind {
                        agenda_parser::event::EventKind::Td(kind) => format!("TD: {}", kind),
                        agenda_parser::event::EventKind::Cm(kind) => format!("CM: {}", kind),
                        agenda_parser::event::EventKind::Tp(kind) => format!("TP: {}", kind),
                        agenda_parser::event::EventKind::Other(kind) => kind.to_string(),
                    };
                    
                    events.push(html! {
                        <div style=format!("background-color: #98fb98; position: absolute; top: {}px; height: {}px;", px_offset, px_height) class="event">
                            <span class="name">{ name }</span>
                            <span class="teacher">{ event.teachers.join(", ") }</span>
                            <span>{"Dumont Durville - B - Rj - 11"}</span>
                            <div class="lesson-details" style="display: none;">
                                <div class="lesson-details-header">
                                    <span>{"01h00 - Lundi 3 janvier"}</span>
                                </div>
                                <div class="lesson-details-content">
                                    
                                </div>
                            </div>
                        </div>
                    });
                }
            }

            day_names.push(html! {
                <span>
                    { format!("{} {} {}", dayname, day, month) }
                </span>
            });
            days.push(html! {
                <div class="day day-mobile-active">
                    { events }
                </div>
            });
        }

        html! {
            <>
            <header>
                <a id="header-logo" href="../index.html">
                <img src="http://localhost:8080/assets/elements/webLogo.svg" alt="INSAgenda logo"/>
                </a>
            </header>
            <section class="section-page-title">
                <h2 class="page-title">{"Mon emploi du temps"}</h2>
                <div class="divider-bar"></div>
            </section>
            <main>
            <div id="calendar">
                <div id="calendar-hours">
                    <span>{"08:00"}</span>
                    <span>{"09:45"}</span>
                    <span>{"11:30"}</span>
                    <span>{"13:15"}</span>
                    <span>{"15:00"}</span>
                    <span>{"16:45"}</span>
                    <span>{"18:30"}</span>
                </div>
                <div id="calendar-main-part">
                    <div id="calendar-top">
                        <a id="calendar-arrow-left"></a>
                        <a id="mobile-day-name">{"Lundi 3 janvier"}</a>
                        <a id="calendar-arrow-right"></a>
                        { day_names }
                    </div>
                    <div id="day-container">
                        <div id="line-container">
                            <div class="line"></div>
                            <div class="line"></div>
                            <div class="line"></div>
                            <div class="line"></div>
                            <div class="line"></div>
                            <div class="line"></div>
                        </div>
                        { days }
                    </div>
                </div>
            </div>
            <div id="option">
                <div id="option-header">
                    <span>{"Options"}</span>
                    <div class="divider-bar-option"></div>
                </div>
                <div id="option-content">
                    <span class="option-name">{"Calendrier :"}</span>
                </div>
                <div id="small-calendar">
                    <div id="calendar-header">
                        <button class="calendar-button" id="calendar-before"></button>
                        <span id="calendar-title">{"Janvier 2022"}</span>
                        <button class="calendar-button" id="calendar-after"></button>
                    </div>
                    <div id="calendar-content">
                        <div id="calendar-days">
                            <span>{"Lun"}</span>
                            <span>{"Mar"}</span>
                            <span>{"Mer"}</span>
                            <span>{"Jeu"}</span>
                            <span>{"Ven"}</span>
                            <span>{"Sam"}</span>
                            <span>{"Dim"}</span>
                        </div>
                        <div id="week1" class="calendar-week">
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                        </div>
                        <div id="week2" class="calendar-week">
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                        </div>
                        <div id="week3" class="calendar-week">
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                        </div>
                        <div id="week4" class="calendar-week">
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                        </div>
                        <div id="week5" class="calendar-week">
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                        </div>
                        <div id="week6" class="calendar-week">
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                            <span class="calendar-case"></span>
                        </div>
                    </div>
                </div>
            </div>
        </main>
            </>
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
