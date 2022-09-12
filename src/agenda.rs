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
        let mobile = crate::slider::width() <= 1000;

        let performance_now = js_sys::Date::now();

        // Go on the first day of the week
        let mut current_day = self.selected_day;
        match mobile {
            true => current_day = current_day.pred().pred(),
            false => for _ in 0..self.selected_day.weekday().num_days_from_monday() {
                current_day = current_day.pred();
            },
        };

        // Check if there is room for the announcement on mobile
        let announcement = self.displayed_announcement.as_ref();
        let mut show_mobile_announcement = mobile && announcement.is_some();
        if show_mobile_announcement {
            let announcement_start = current_day.succ().succ().and_hms(18,30,0).timestamp() as u64;
            let announcement_end = current_day.succ().succ().and_hms(20,0,0).timestamp() as u64;
            let announcement_range = announcement_start..=announcement_end;
            match self.events.binary_search_by_key(&announcement_start, |e| e.start_unixtime) { // Check if an event starts exactly at that time.
                Ok(_) => {
                    show_mobile_announcement = false;
                },
                Err(mut idx) => {
                    if let Some(e) = self.events.get(idx) {
                        if announcement_range.contains(&(e.start_unixtime)) { // Check if the next event starts in the range
                            show_mobile_announcement = false;
                        } else {
                            idx -= 1;
                            while let Some(e) = self.events.get(idx) { // Check if a few previous events end in the range
                                if announcement_range.contains(&(e.end_unixtime)) {
                                    show_mobile_announcement = false;
                                    break;
                                }
                                if e.end_unixtime < announcement_start - 6*3600 { // Avoid backtracking too much, 6h is enough
                                    break;
                                }
                                idx -= 1;
                            }
                        }
                    }
                },
            };
        }
        let agenda_class = if show_mobile_announcement { "show-announcement" } else { "" };
        let announcement = announcement.map(|a| view_announcement(a, ctx));

        // Build each day and put events in them
        let mut days = Vec::new();
        let mut day_names = Vec::new();
        for d in 0..5 {
            let mut events = Vec::new();

            // Iterate over events, starting from the first one that starts during the current day
            let day_start = current_day.and_hms(0,0,0).timestamp() as u64;
            let mut idx = match self.events.binary_search_by_key(&day_start, |e| e.start_unixtime) {
                Ok(idx) => idx,
                Err(idx) => idx,
            };
            while let Some(e) = self.events.get(idx) {
                if e.start_unixtime > day_start + 24*3600 {
                    break;
                }
                events.push(html! {
                    <EventComp
                        day_of_week={d}
                        event={e.clone()}
                        day_start={current_day.and_hms(0,0,0).timestamp() as u64}
                        app_link={ctx.link().clone()}
                        show_announcement={show_mobile_announcement}>
                    </EventComp>
                });
                idx += 1;
            }

            let day_style = match mobile {
                true => format!("position: absolute; left: {}%;", (current_day.num_days_from_ce()-730000) * 20),
                false => String::new(),
            };

            day_names.push(html! {
                <span id={if current_day == self.selected_day {"selected-day"} else {""}}>
                    { format_day(current_day.weekday(), current_day.day(), current_day.month()) }
                </span>
            });
            days.push(html! {
                <div class="day" style={day_style}>
                    { events }
                </div>
            });

            current_day = current_day.succ();
        }

        let elapsed = js_sys::Date::now() - performance_now;
        log!("Rendered agenda in {}ms", elapsed);

        html! {
            <>
            <header>
                <a id="header-logo" href="/agenda">
                <img src="/assets/logo/logo.svg" alt="INSAgenda logo"/> 
                <h1 id="header-name">{"INSAgenda"}</h1>
                </a>
                <button id="settings-button" onclick={ctx.link().callback(|_| AppMsg::SetPage(Page::Settings))}/>
            </header>
            <main id="agenda-main" class={agenda_class}>
            <div id="agenda">
                <div id="agenda-hours">
                    <span>{"08:00"}</span>
                    <span>{"09:45"}</span>
                    <span>{"11:30"}</span>
                    <span>{"13:15"}</span>
                    <span>{"15:00"}</span>
                    <span>{"16:45"}</span>
                    if !show_mobile_announcement {<span>{"18:30"}</span>}
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
                        <div id="day-container" style={if mobile {Some(format!("position: relative; right: {}%", 100 * (self.selected_day.num_days_from_ce() - 730000)))} else {None}}>
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
                <Calendar day={self.selected_day.day()} month={self.selected_day.month()} year={self.selected_day.year()} app_link={ctx.link().clone()}/>
                if !mobile {
                    if let Some(announcement) = announcement.clone() {
                        { announcement }
                    }
                }
                <br/>
            </div>
            if mobile && show_mobile_announcement {
                if let Some(announcement) = announcement {
                    { announcement }
                }
            }
        </main>
            </>
        }
    }
}
