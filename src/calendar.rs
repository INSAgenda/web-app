use crate::{prelude::*, slider::width};

#[derive(Clone, Properties)]
pub struct CalendarProps {
    pub agenda_link: Scope<Agenda>,
    pub day: u32,
    pub month: u32,
    pub year: i32,
}

impl PartialEq for CalendarProps {
    fn eq(&self, _other: &Self) -> bool { true }
}

pub enum Msg {
    NextMonth,
    PreviousMonth,
    NextWeek,
    PreviousWeek,
    Goto { day: u32, month: u32, year: i32 },
    TriggerFold,
}

pub struct Calendar {
    selected_day: u32,
    selected_month: u32,
    selected_year: i32,
    folded: bool,
}

impl Component for Calendar {
    type Message = Msg;
    type Properties = CalendarProps;

    fn create(ctx: &Context<Self>) -> Self {
        Calendar {
            selected_day: ctx.props().day,
            selected_month: ctx.props().month,
            selected_year: ctx.props().year as i32,
            folded: true,
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
                self.selected_day = 1;
                ctx.props().agenda_link.send_message(AgendaMsg::Goto {
                    day: self.selected_day,
                    month: self.selected_month,
                    year: self.selected_year
                });

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
                self.selected_day = last_day.day();
                ctx.props().agenda_link.send_message(AgendaMsg::Goto {
                    day: self.selected_day,
                    month: self.selected_month,
                    year: self.selected_year
                });

                true
            },
            Msg::NextWeek => {
                let next_week = NaiveDateTime::new(NaiveDate::from_ymd(self.selected_year, self.selected_month, self.selected_day), NaiveTime::from_hms(0, 0, 0)) + chrono::Duration::days(7);
                ctx.link().send_message(Msg::Goto {
                    day: next_week.day(),
                    month: next_week.month(),
                    year: next_week.year() as i32
                });
                log!("next week: {:?}", next_week);
                false
            },
            Msg::PreviousWeek => {
                let previous_week = NaiveDateTime::new(NaiveDate::from_ymd(self.selected_year, self.selected_month, self.selected_day), NaiveTime::from_hms(0, 0, 0)) - chrono::Duration::days(7);
                ctx.link().send_message(Msg::Goto {
                    day: previous_week.day(),
                    month: previous_week.month(),
                    year: previous_week.year() as i32
                });
                log!("previous week: {:?}", previous_week);
                false
            }
            Msg::Goto { day, month, year } => {
                self.selected_day = day;
                self.selected_month = month;
                self.selected_year = year;
                ctx.props().agenda_link.send_message(AgendaMsg::Goto {day, month,year});
                true
            },
            Msg::TriggerFold => {
                self.folded = !self.folded;
                true
            } 
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let display_month = format!("{} {}", t(match self.selected_month {
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
        }), self.selected_year);

        if self.folded {
            html!{
                <div class="calendar">
                    <div class="calendar-header">
                        <button class="calendar-arrow" onclick={ctx.link().callback(|_| Msg::PreviousWeek)}></button>

                        <img src="/agenda/images/calendar-btn.svg" onclick={ctx.link().callback(|_| Msg::TriggerFold)}/>
                        <span id="calendar-title">{display_month}</span>

                        <button class="calendar-arrow" id="calendar-arrow-right" onclick={ctx.link().callback(|_| Msg::NextWeek)}></button>
                    </div>
                </div>
            }
        } else {
            let first_day = NaiveDate::from_ymd(self.selected_year, self.selected_month, 1);
            let last_day = NaiveDate::from_ymd(self.selected_year, (self.selected_month % 12) + 1, 1).pred();

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
                <div id="calendar">
                    <div id="calendar-header">
                        <button class="calendar-arrow" onclick={ctx.link().callback(|_| Msg::PreviousMonth)}></button>
                        <span id="calendar-title">{display_month}</span>
                        <button class="calendar-arrow" onclick={ctx.link().callback(|_| Msg::NextMonth)} id="calendar-right-arrow"></button>
                    </div>
                    <div id="calendar-content">
                        <div id="calendar-days">
                            <span>{t("Lun")}</span>
                            <span>{t("Mar")}</span>
                            <span>{t("Mer")}</span>
                            <span>{t("Jeu")}</span>
                            <span>{t("Ven")}</span>
                            <span>{t("Sam")}</span>
                            <span>{t("Dim")}</span>
                        </div>
                        { weeks }
                    </div>
                </div>
            }

        }
    }
}
