use yew::prelude::*;
use chrono::{Datelike, FixedOffset, Local, NaiveDate};
use crate::log;

pub struct Msg {

}

pub struct Calendar {
    selected_day: u32,
    selected_month: u32,
    selected_year: i32,
    link: ComponentLink<Self>,
}

impl Component for Calendar {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let now = Local::now();
        let now = now.with_timezone(&FixedOffset::east(1 * 3600));

        Calendar {
            selected_day: now.day(),
            selected_month: now.month(),
            selected_year: now.year(),
            link
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender { false }

    fn view(&self) -> Html {
        let mut date = NaiveDate::from_ymd(self.selected_year, self.selected_month, self.selected_day);
        let first_day = NaiveDate::from_ymd(self.selected_year, self.selected_month, 1);
        let last_day = NaiveDate::from_ymd(self.selected_year, (self.selected_month % 12) + 1, 1).pred();
        log!("day count: {}", last_day.day());

        let mut calendar_cases = Vec::new();
        for _ in 0..first_day.weekday().number_from_monday() - 1 {
            calendar_cases.push(html! {
                <span class="calendar-case"></span>
            });
        }

        for day in 1..=last_day.day() {
            calendar_cases.push(html! {
                <span class="calendar-case">{day.to_string()}</span>
            });
        }

        while calendar_cases.len() % 7 != 0 {
            calendar_cases.push(html! {
                <span class="calendar-case"></span>
            });
        }

        let mut weeks = Vec::new();
        for week in 1..=calendar_cases.len()/7 {
            weeks.push(html! {
                <div id=format!("week{}", week) class="calendar-week">
                    { calendar_cases.drain(0..7).collect::<Vec<_>>() }
                </div>
            })
        }

        html! {
            <div id="small-calendar">
                <div id="calendar-header">
                    <button class="calendar-arrow"></button>
                    <span id="calendar-title">{"Janvier 2022"}</span>
                    <button class="calendar-arrow" id="calendar-right-arrow"></button>
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
                    { weeks }
                </div>
            </div>
        }
    }
}
