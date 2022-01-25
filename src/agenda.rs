use chrono::{Weekday, Datelike, TimeZone};
use chrono_tz::Europe::Paris;
use crate::{event::EventComp, App, calendar::Calendar};
use yew::prelude::*;

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
    pub fn view_agenda(&self, ctx: &Context<Self>) -> Html {
        let selected_day_dt = Paris.ymd(self.selected_day.2, self.selected_day.1, self.selected_day.0);

        // Go on the first day of the week
        let mut current_day = selected_day_dt;
        for _ in 0..selected_day_dt.weekday().num_days_from_monday() {
            current_day = current_day.pred();
        }

        let mut days = Vec::new();
        let mut day_names = Vec::new();
        for _ in 0..5 {
            let mut events = Vec::new();
            for event in &self.events {
                if (event.start_unixtime as i64) > current_day.and_hms(0,0,0).timestamp()
                    && (event.start_unixtime as i64) < current_day.and_hms(23,59,59).timestamp()
                {
                    events.push(html! {
                        <EventComp event={event.clone()} day_start={current_day.and_hms(0,0,0).timestamp() as u64} global={self.event_global.clone()}></EventComp>
                    });
                }
            }

            day_names.push(html! {
                <span id={if current_day == selected_day_dt {"selected-day"} else {""}}>
                    { format_day(current_day.weekday(), current_day.day(), current_day.month()) }
                </span>
            });
            days.push(html! {
                <div class="day">
                    { events }
                </div>
            });

            current_day = current_day.succ();
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
                        <a id="calendar-arrow-left" onclick={ctx.link().callback(|_| crate::Msg::Previous)}></a>
                        { day_names }
                        <a id="calendar-arrow-right" onclick={ctx.link().callback(|_| crate::Msg::Next)}></a>
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
                <Calendar app_link={ctx.link().clone()}/>
                <br/>
                <div class="white-button" onclick={ctx.link().callback(|_| crate::Msg::SetPage(crate::Page::Settings))}>{"Paramètres"}</div>    
            </div>
        </main>
            </>
        }
    }
}
