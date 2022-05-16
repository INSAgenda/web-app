use crate::prelude::*;

fn format_day(day_name: Weekday, day: u32, month: u32) -> String {
    let month = t(match month {
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
    });

    let day_name = t(match day_name {
        Weekday::Mon => "Lundi",
        Weekday::Tue => "Mardi",
        Weekday::Wed => "Mercredi",
        Weekday::Thu => "Jeudi",
        Weekday::Fri => "Vendredi",
        Weekday::Sat => "Samedi",
        Weekday::Sun => "Dimanche",
    });

    format!("{} {} {}", day_name, day, month)
}

impl App {
    pub fn view_agenda(&self, ctx: &Context<Self>) -> Html {
        let mobile_view = crate::slider::width() <= 1000;

        // Go on the first day of the week
        let mut current_day = self.selected_day;
        match mobile_view {
            true => current_day = current_day.pred().pred(),
            false => for _ in 0..self.selected_day.weekday().num_days_from_monday() {
                current_day = current_day.pred();
            },
        };

        let mut days = Vec::new();
        let mut day_names = Vec::new();
        for d in 0..5 {
            let mut events = Vec::new();
            for event in &self.events {
                if (event.start_unixtime as i64) > current_day.and_hms(0,0,0).timestamp()
                    && (event.start_unixtime as i64) < current_day.and_hms(23,59,59).timestamp()
                {
                    events.push(html! {
                        <EventComp day_of_week={d} event={event.clone()} day_start={current_day.and_hms(0,0,0).timestamp() as u64} app_link={ctx.link().clone()}></EventComp>
                    });
                }
            }

            day_names.push(html! {
                <span id={if current_day == self.selected_day {"selected-day"} else {""}}>
                    { format_day(current_day.weekday(), current_day.day(), current_day.month()) }
                </span>
            });
            days.push(html! {
                <div class="day" style={if mobile_view {Some(format!("position: absolute; left: {}%;", (current_day.num_days_from_ce()-730000) * 20))} else {None}}>
                    { events }
                </div>
            });

            current_day = current_day.succ();
        }

        html! {
            <>
            <header>
                <a id="header-logo" href="../index.html">
                <img src="/assets/logo/logo.svg" alt="INSAgenda logo"/> 
                <h1 id="header-name">{"INSAgenda"}</h1>
                </a>
                <button id="settings-button" onclick={ctx.link().callback(|_| AppMsg::SetPage(Page::Settings))}/>
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
                        <a id="agenda-arrow-left" onclick={ctx.link().callback(|_| AppMsg::Previous)}>
                            <div></div>
                        </a>
                        { day_names }
                        <a id="agenda-arrow-right" onclick={ctx.link().callback(|_| AppMsg::Next)}>
                            <div></div>
                        </a>
                    </div>
                    <div id="day-container-scope">
                        <div id="day-container" style={if mobile_view {Some(format!("position: relative; right: {}%", 100 * (self.selected_day.num_days_from_ce() - 730000)))} else {None}}>
                            { days }
                        </div>
                    </div>
                </div>
            </div>
            <div id="option">
                <div id="option-header">
                    <span>{t("Options")}</span>
                    <div class="divider-bar-option"></div>
                </div>
                <Calendar app_link={ctx.link().clone()}/>
                <br/>
            </div>
        </main>
            </>
        }
    }
}
