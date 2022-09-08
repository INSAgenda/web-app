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

        let announcement = self.displayed_announcement.as_ref();
        let mut show_announcement = mobile_view && announcement.is_some();

        // Go on the first day of the week
        let mut current_day = self.selected_day;
        match mobile_view {
            true => current_day = current_day.pred().pred(),
            false => for _ in 0..self.selected_day.weekday().num_days_from_monday() {
                current_day = current_day.pred();
            },
        };

        // Check if there is room for the mobile ad TODO if mobile
        let announcement_start = current_day.succ().succ().and_hms(18,30,0).timestamp();
        let announcement_end = current_day.succ().succ().and_hms(20,0,0).timestamp();
        let announcement_range = announcement_start..=announcement_end;
        for event in &self.events {
            if announcement_range.contains(&(event.start_unixtime as i64)) || announcement_range.contains(&(event.end_unixtime as i64)) {
                show_announcement = false;
                break;
            }
        }

        let mut days = Vec::new();
        let mut day_names = Vec::new();
        for d in 0..5 {
            let mut events = Vec::new();
            for event in &self.events {
                if (event.start_unixtime as i64) > current_day.and_hms(0,0,0).timestamp()
                    && (event.start_unixtime as i64) < current_day.and_hms(23,59,59).timestamp()
                {
                    events.push(html! {
                        <EventComp
                            day_of_week={d}
                            event={event.clone()}
                            day_start={current_day.and_hms(0,0,0).timestamp() as u64}
                            app_link={ctx.link().clone()}
                            show_announcement={show_announcement}>
                        </EventComp>
                    });
                }
            }

            let mut day_style = match mobile_view {
                true => format!("position: absolute; left: {}%;", (current_day.num_days_from_ce()-730000) * 20),
                false => String::new(),
            };
            /*if show_announcement {
                day_style.push_str("height: calc(100% / 43200 * (43200 - 6300));");
            }*/

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

        // Build announcement
        let announcement = announcement.map(|announcement| {
            match announcement.ty {
                ContentType::Text => html! {
                    <div id="announcement" class="announcement-default-style">
                        <div>{announcement.title.as_str()}</div>
                        <p>
                            {match SETTINGS.lang() {
                                Lang::French => announcement.content_fr.as_deref().unwrap_or_default(),
                                Lang::English => announcement.content_en.as_deref().unwrap_or_default(),
                            }}
                        </p>
                    </div>
                }, // TODO lang
                ContentType::Html => {
                    let div = window()
                        .doc()
                        .create_element("div")
                        .unwrap();
                    match SETTINGS.lang() {
                        Lang::French => div.set_inner_html(announcement.content_fr.as_deref().unwrap_or_default()),
                        Lang::English => div.set_inner_html(announcement.content_en.as_deref().unwrap_or_default()),
                    }
                    let node = web_sys::Node::from(div);
                    let vnode = yew::virtual_dom::VNode::VRef(node);
                    html! {
                        <div id="announcement">
                            {vnode}
                        </div>
                    }
                }
            }
        });

        let agenda_class = if show_announcement { "show-announcement" } else { "" };

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
                    if !show_announcement {<span>{"18:30"}</span>}
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
                <Calendar day={self.selected_day.day()} month={self.selected_day.month()} year={self.selected_day.year()} app_link={ctx.link().clone()}/>
                if !mobile_view {
                    if let Some(announcement) = announcement.clone() {
                        { announcement }
                    }
                }
                <br/>
            </div>
            if mobile_view && show_announcement {
                if let Some(announcement) = announcement {
                    { announcement }
                }
            }
        </main>
            </>
        }
    }
}
