use chrono::{offset::FixedOffset, Weekday, Datelike, TimeZone};
use crate::{event::EventComp, App};
use yew::{
    prelude::*,
};

impl App {
    pub fn view_agenda(&self) -> Html {
        let mut days = Vec::new();
        let mut day_names = Vec::new();
        for offset in 0..5 {
            let datetime =
                FixedOffset::east(1 * 3600).timestamp((self.weekstart + offset * 86400) as i64, 0);
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
                    events.push(html! {
                        <EventComp event=event.clone() day_start={self.weekstart+offset*86400} global=self.event_global.clone()></EventComp>
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
                <h1 class="page-title">{"Mon emploi du temps"}</h1>
                <a id="header-logo" href="../index.html">
                <img src="http://localhost:8080/assets/elements/webLogo.svg" alt="INSAgenda logo"/>
                </a>
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
                        <a id="calendar-arrow-left"></a>
                        <a id="mobile-day-name">{"Lundi 3 janvier"}</a>
                        <a id="calendar-arrow-right"></a>
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
                <div class="option-name big-white-button" onclick=self.link.callback(|_| crate::Msg::SetPage(crate::Page::Settings))>{"Paramètres"}</div>    
            </div>
        </main>
            </>
        }
    }
}
