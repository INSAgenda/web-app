use chrono::{offset::FixedOffset, Weekday, Datelike, TimeZone};
use crate::{event::EventComp, App};
use yew::{
    prelude::*,
};

fn format_day(day_name: Weekday, day: u32, month: u32) -> String {
    let month = match month {
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

    let day_name = match day_name {
        Weekday::Mon => "Lundi",
        Weekday::Tue => "Mardi",
        Weekday::Wed => "Mercredi",
        Weekday::Thu => "Jeudi",
        Weekday::Fri => "Vendredi",
        Weekday::Sat => "Samedi",
        Weekday::Sun => "Dimanche",
    };

    format!("{} {} {}", day_name, day, month)
}

impl App {
    pub fn view_agenda(&self) -> Html {
        let selected_day_datetime = FixedOffset::east(1 * 3600).timestamp(self.day_start as i64, 0);
        let selected_day = format_day(selected_day_datetime.weekday(), selected_day_datetime.day(), selected_day_datetime.month());

        let mut days = Vec::new();
        let mut day_names = Vec::new();
        for offset in 0..5 {
            let datetime =
                FixedOffset::east(1 * 3600).timestamp((self.week_start() + offset * 86400) as i64, 0);
            let is_selected_day = datetime.timestamp() as u64 == self.day_start;

            let mut events = Vec::new();
            for event in &self.events {
                if (event.start_unixtime as i64) > datetime.timestamp()
                    && (event.start_unixtime as i64) < datetime.timestamp() + 86400
                {
                    events.push(html! {
                        <EventComp event=event.clone() day_start={self.week_start()+offset*86400} global=self.event_global.clone()></EventComp>
                    });
                }
            }

            day_names.push(html! {
                <span>
                    { format_day(datetime.weekday(), datetime.day(), datetime.month()) }
                </span>
            });
            days.push(html! {
                <div class=if is_selected_day {"day selected-day"} else {"day"}>
                    { events }
                </div>
            });
        }

        html! {
            <>
            <header>
                <h1 class="page-title">{"Mon emploi du temps"}</h1>
                <div id="header-logo"><img src="../assets/logo/logo.svg"/></div>
            </header>
            <main id="calendar-main">
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
                        <a id="calendar-arrow-left" onclick=self.link.callback(|_| crate::Msg::Previous)></a>
                        <a id="mobile-day-name">{selected_day}</a>
                        <a id="calendar-arrow-right" onclick=self.link.callback(|_| crate::Msg::Next)></a>
                        { day_names }
                    </div>
                    <div id="day-container">
                        <div id="line-container">
                            <div class="line"><div></div></div>
                            <div class="line"><div></div></div>
                            <div class="line"><div></div></div>
                            <div class="line"><div></div></div>
                            <div class="line"><div></div></div>
                            <div class="line"><div></div></div>
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
                <div class="option-name">{"Calendrier :"}</div>
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
                <br/>
                <div class="white-button" onclick=self.link.callback(|_| crate::Msg::SetPage(crate::Page::Settings))>{"Paramètres"}</div>    
            </div>
        </main>
            </>
        }
    }
}
