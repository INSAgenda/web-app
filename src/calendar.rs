use yew::prelude::*;
use chrono::{Datelike, Local, NaiveDate};
use chrono_tz::Europe::Paris;

#[derive(Clone, Properties)]
pub struct CalendarProps {
    pub app_link: yew::html::Scope<crate::App>,
}

impl PartialEq for CalendarProps {
    fn eq(&self, _other: &Self) -> bool { true }
}

pub enum Msg {
    NextMonth,
    PreviousMonth,
    Goto { day: u32, month: u32, year: i32 },
}

pub struct Calendar {
    selected_day: u32,
    selected_month: u32,
    selected_year: i32
}

impl Component for Calendar {
    type Message = Msg;
    type Properties = CalendarProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let now = Local::now();
        let now = now.with_timezone(&Paris);

        Calendar {
            selected_day: now.day(),
            selected_month: now.month(),
            selected_year: now.year()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NextMonth => {
                if self.selected_month == 12 {
                    self.selected_month = 1;
                    self.selected_year += 1;
                } else {
                    self.selected_month += 1;
                }
                let last_day = NaiveDate::from_ymd(self.selected_year, (self.selected_month % 12) + 1, 1).pred();
                if self.selected_day > last_day.day() {
                    self.selected_day = last_day.day();
                }

                true
            },
            Msg::PreviousMonth => {
                if self.selected_month == 1 {
                    self.selected_month = 12;
                    self.selected_year -= 1;
                } else {
                    self.selected_month -= 1;
                }
                let last_day = NaiveDate::from_ymd(self.selected_year, (self.selected_month % 12) + 1, 1).pred();
                if self.selected_day > last_day.day() {
                    self.selected_day = last_day.day();
                }

                true
            },
            Msg::Goto { day, month, year } => {
                self.selected_day = day;
                self.selected_month = month;
                self.selected_year = year;
                ctx.props().app_link.send_message(crate::Msg::Goto {day,month,year});
                true
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let first_day = NaiveDate::from_ymd(self.selected_year, self.selected_month, 1);
        let last_day = NaiveDate::from_ymd(self.selected_year, (self.selected_month % 12) + 1, 1).pred();

        let display_month = format!("{} {}", match self.selected_month {
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
        }, self.selected_year);

        let mut calendar_cases = Vec::new();
        for _ in 0..first_day.weekday().number_from_monday() - 1 {
            calendar_cases.push(html! {
                <span class="calendar-case" onclick={ctx.link().callback(|_| Msg::PreviousMonth)}></span>
            });
        }

        for day in 1..=last_day.day() {
            let month = self.selected_month;
            let year = self.selected_year;
            calendar_cases.push(html! {
                <span class="calendar-case" id={if day==self.selected_day {Some("selected-calendar-case")} else {None}} onclick={ctx.link().callback(move |_| Msg::Goto {day,month,year})}>{day.to_string()}</span>
            });
        }

        while calendar_cases.len() % 7 != 0 {
            calendar_cases.push(html! {
                <span class="calendar-case" onclick={ctx.link().callback(|_| Msg::NextMonth)}></span>
            });
        }

        let mut weeks = Vec::new();
        for week in 1..=calendar_cases.len()/7 {
            weeks.push(html! {
                <div id={format!("week{}", week)} class="calendar-week">
                    { calendar_cases.drain(0..7).collect::<Vec<_>>() }
                </div>
            })
        }

        html! {
            <div id="small-calendar">
                <div id="calendar-header">
                    <button class="calendar-arrow" onclick={ctx.link().callback(|_| Msg::PreviousMonth)}></button>
                    <span id="calendar-title">{display_month}</span>
                    <button class="calendar-arrow" onclick={ctx.link().callback(|_| Msg::NextMonth)} id="calendar-right-arrow"></button>
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
