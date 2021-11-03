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
    events: Vec<Event>,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let date = chrono::Local::now();
        let date = date.with_timezone(&chrono::offset::FixedOffset::east(2 * 3600));

        let daystart = date.timestamp() - (date.timestamp() + 2 * 3600) % 86400;
        let mut weekstart = daystart - (date.weekday().number_from_monday() as i64 - 1) * 86400;
        if date.weekday().number_from_monday() >= 6 {
            weekstart += 7 * 86400;
        }

        let mut app = Self {
            weekstart,
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
        for offset in 0..6 {
            let datetime =
                FixedOffset::east(2 * 3600).timestamp(self.weekstart + offset * 86400, 0);
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
                    let start_time = FixedOffset::east(2 * 3600).timestamp(event.start_unixtime as i64, 0);
                    let end_time = FixedOffset::east(2 * 3600).timestamp(event.end_unixtime as i64, 0);
                    let start_time_is_common = [(8,0), (9,30), (9,45), (11,15), (11,30), (13,0), (15,0), (16,30), (16,45), (18,15)]
                        .contains(&(start_time.hour(), start_time.minute()));
                    let end_time_is_common = [(8,0), (9,30), (9,45), (11,15), (11,30), (13,0), (15,0), (16,30), (16,45), (18,15)]
                        .contains(&(end_time.hour(), end_time.minute()));
                    let times_are_common = start_time_is_common && end_time_is_common;
                    
                    events.push(html! {
                        <div>
                            {
                                match times_are_common {
                                    true => html!(),
                                    false => html! {<>
                                        {format!("{}h{} - {}h{}", start_time.hour(), start_time.minute(), end_time.hour(), end_time.minute())}
                                        <br/> 
                                    </>},
                                }
                            }
                            {format!("{:?}", event.kind)}
                        </div>
                    });
                }
            }

            days.push(html! {
                <div>
                    { format!("{} {} {}", dayname, day, month) }
                    { events }
                </div>
            });
        }

        html! {
            <main>
                <button onclick=self.link.callback(|_| Msg::PreviousWeek)>{"Semaine précédente"}</button>
                <div id="days">
                    { days }
                </div>
                <button onclick=self.link.callback(|_| Msg::NextWeek)>{"Semaine suivante"}</button>
            </main>
        }
    }
}

impl App {
    fn new_fetch_task(&mut self, time_range: std::ops::Range<i64>) {
        let request = Request::get(format!(
            "http://127.0.0.1:8080/api/schedule/Stpi1/E-1/ALL/{}-{}",
            time_range.start, time_range.end
        ))
        .body(Nothing)
        .expect("Could not build request.");

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
