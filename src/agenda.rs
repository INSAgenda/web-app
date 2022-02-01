use chrono::{Weekday, Datelike};
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
        // Go on the first day of the week
        let mut current_day = self.selected_day;
        for _ in 0..self.selected_day.weekday().num_days_from_monday() {
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
                        <EventComp event={event.clone()} day_start={current_day.and_hms(0,0,0).timestamp() as u64} app_link={ctx.link().clone()}></EventComp>
                    });
                }
            }

            day_names.push(html! {
                <span id={if current_day == self.selected_day {"selected-day"} else {""}}>
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
                <a id="header-logo" href="../index.html">
                <img src="/assets/elements/webLogo.svg" alt="INSAgenda logo"/> 
                <span id="header-name">{"INSAgenda"}</span>
                </a>
            </header>
            <main id="agenda-main">
            <div id="agenda">
                <div id="agenda-hours">
                    <span>{"08:00"}</span>
                    <span>{"09:45"}</span>
                    <span>{"11:30"}</span>
                    <span>{"13:15"}</span>
                    <span>{"15:00"}</span>
                    <span>{"16:45"}</span>
                    <span>{"18:30"}</span>
                </div>
                <div id="agenda-main-part">
                    <div id="agenda-top">
                        <a id="agenda-arrow-left" onclick={ctx.link().callback(|_| crate::Msg::Previous)}>
                            <div></div>
                        </a>
                        { day_names }
                        <a id="agenda-arrow-right" onclick={ctx.link().callback(|_| crate::Msg::Next)}>
                            <div></div>
                        </a>
                    </div>
                    <div id="day-container-scope">
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
            </div>
            <div id="option">
                <div id="option-header">
                    <span>{"Options"}</span>
                    <div class="divider-bar-option"></div>
                </div>
                <Calendar app_link={ctx.link().clone()}/>
                <br/>
                <div class="white-button" onclick={ctx.link().callback(|_| crate::Msg::SetPage(crate::Page::Settings))}>{"Paramètres"}</div>    
            </div>
        </main>
            </>
        }
    }
}
