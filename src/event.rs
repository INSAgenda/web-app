use crate::{prelude::*, slider::width};

lazy_static::lazy_static!{
    static ref ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Properties, Clone)]
pub struct EventCompProps {
    pub event: RawEvent,
    pub day_start: u64,
    pub has_mobile_ad: bool,
    pub app_link: yew::html::Scope<crate::App>,
    pub day_of_week: u8,
}

impl PartialEq for EventCompProps {
    fn eq(&self, other: &Self) -> bool {
        !COLORS_CHANGED.load(Ordering::Relaxed) && self.event.start_unixtime == other.event.start_unixtime && self.event.end_unixtime == other.event.end_unixtime // TODO: add other fields
    }
}

pub struct EventComp {
    popup_displayed: bool,
    on_click: Closure<dyn FnMut(web_sys::MouseEvent)>,
    last_click_timestamp: i64,
    popup_id: String,
}

pub enum EventCompMsg {
    ShowPopup(bool),
    SaveColors,
}

impl Component for EventComp {
    type Message = EventCompMsg;
    type Properties = EventCompProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = format!("event-popup-{}", ID_COUNTER.fetch_add(1, Ordering::Relaxed));

        // Creates a closure called on click that will close the popup if the user clicked outside of it
        let doc = window().doc();
        let id2 = id.clone();
        let link = ctx.link().clone();
        let on_click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let popup_el = doc.get_element_by_id(&id2).unwrap();
            let rect = popup_el.get_bounding_client_rect();

            // Check the click was not inside the popup
            if (rect.y()..rect.y()+rect.height()).contains(&(event.client_y() as f64))
                && (rect.x()..rect.x()+rect.width()).contains(&(event.client_x() as f64))
            { return; }
            link.send_message(EventCompMsg::ShowPopup(false));

        }) as Box<dyn FnMut(_)>);

        EventComp {
            popup_displayed: false,
            on_click,
            last_click_timestamp: 0,
            popup_id: id,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EventCompMsg::ShowPopup(state) => {
                // Double call protection
                let now = chrono::Utc::now().timestamp_millis();
                if self.last_click_timestamp + 100 > now {
                    return false;
                }
                self.last_click_timestamp = now;

                self.popup_displayed = state;
                if self.popup_displayed {
                    window().add_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();
                } else {
                    window().remove_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();
                }

                // Enable/Disable slider 
                if width() <= 1000 {
                    ctx.props().app_link.send_message(AppMsg::SetSliderState(!self.popup_displayed));
                }

                true
            },
            EventCompMsg::SaveColors => {
                let document = window().doc();
                let background_color = match document.query_selector(&format!("#{} #background-color-input", self.popup_id)).unwrap() {
                    Some(el) => el.dyn_into::<HtmlInputElement>().unwrap().value(),
                    None => return false,
                };

                COLORS.set(&ctx.props().event.summary, background_color); 

                // We need to set this so that other events know that they have to refresh
                COLORS_CHANGED.store(true, Ordering::Relaxed);

                ctx.props().app_link.send_message(crate::Msg::Refresh);

                false
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Format title
        let summary = &ctx.props().event.summary;
        let name = match &ctx.props().event.kind {
            Some(EventKind::Td) => format!("TD: {summary}"),
            Some(EventKind::Tp) => format!("TP: {summary}"),
            Some(EventKind::Cm) => format!("CM: {summary}"),
            None => summary.clone(),
        };
        let bg_color = COLORS.get(&ctx.props().event.summary);
        
        // Format location
        let location = ctx.props().event.location.as_ref().map(|location| {
            match location {
                Location::Parsed { building, building_area, level, room_number } => {
                    let building = match SETTINGS.building_naming() {
                        BuildingNaming::Short => match building {
                            Building::Magellan => "Ma",
                            Building::DumontDurville => "Du",
                            Building::Bougainville => "Bo",
                            Building::Darwin => "Da",
                        },
                        BuildingNaming::Long => match building {
                            Building::Magellan => "Magellan",
                            Building::DumontDurville => "Dumont Durville",
                            Building::Bougainville => "Bougainville",
                            Building::Darwin => "Darwin",
                        },
                    };
                    format!("{} - {} - {} - {}", building, building_area, level, room_number)
                }
                Location::Unparsed(location) => location.clone(),
            }
        });

        // Calculate position
        let day_sec_count = match ctx.props().has_mobile_ad {
            false => 43200.0,
            true => 43200.0 - 6300.0,
        };
        let sec_offset = ctx.props().event.start_unixtime.saturating_sub(ctx.props().day_start + 8 * 3600);
        let percent_offset = 100.0 / day_sec_count * sec_offset as f64;
        if ctx.props().event.start_unixtime >= ctx.props().event.end_unixtime {
            log!("Event {} in {:?}  ends before it starts", name, location);
            return html!{};
        }
        let percent_height = 100.0 / day_sec_count * (ctx.props().event.end_unixtime - ctx.props().event.start_unixtime) as f64;        
    
        let start = Paris.timestamp(ctx.props().event.start_unixtime as i64, 0);
        let end = Paris.timestamp(ctx.props().event.end_unixtime as i64, 0);
        let class = String::new() + if self.popup_displayed {""} else {"hide"};
        let mobile = width() <= 1000;

        let popup = html! {
            <div class={format!("event-details {}", class)} id={self.popup_id.clone()} style={String::new() + if ctx.props().day_of_week > 2 { "left" } else { "right" } + ": -214px;" + if percent_offset > 50. && !mobile {"transform: translateY(-70%);"}  else {""}}  >
                if mobile {
                    <div class="close-arrow" onclick={ Some(ctx.link().callback(|_| EventCompMsg::ShowPopup(false))) } >
                        <svg width="110" height="28" viewBox="0 0 110 28" fill="none" xmlns="http://www.w3.org/2000/svg">
                            <path d="M55.5 28C55.5 28 19.6743 2 0.5 0H55.5V28Z" fill="var(--day)"/>
                            <path d="M55 28C55 28 90.8257 2 110 0H55V28Z" fill="var(--day)"/>
                            <g clip-path="url(#clip0_211_6147)">
                            <path d="M55 18.0015C55.4045 18.0017 55.8051 17.9605 56.1787 17.8802C56.5524 17.7999 56.8918 17.6822 57.1774 17.5337L76.0974 7.72639C76.6752 7.42608 76.9993 7.0191 76.9982 6.59497C76.9971 6.17084 76.6709 5.7643 76.0915 5.4648C75.5121 5.16529 74.7268 4.99736 73.9084 4.99793C73.0901 4.9985 72.3057 5.16752 71.7278 5.46783L54.9883 14.1436L38.2487 5.46783C37.6708 5.16833 36.887 5.00007 36.0698 5.00007C35.2525 5.00007 34.4688 5.16833 33.8909 5.46783C33.313 5.76734 32.9883 6.17355 32.9883 6.59711C32.9883 7.02067 33.313 7.42689 33.8909 7.72639L52.8108 17.5322C53.0976 17.6817 53.4388 17.8001 53.8146 17.8807C54.1904 17.9612 54.5933 18.0023 55 18.0015Z" fill="var(--text)"/>
                            </g>
                        </svg>
                    </div>
                }
                <h3>{ &name }</h3>
                <div style={format!("background-color: {};", bg_color.clone())} class="divider-bar-option"></div>
                <div class="event-details-content">
                    <div class="info-block">
                        <h4>{t("Horaires")}</h4>
                        <span>{format!("{} - {}", start.time().format("%Hh%M"), end.time().format("%Hh%M"))}</span>
                    </div>

                    if let Some(location) = &location { 
                        <div class="info-block">
                            <h4>{t("Emplacement")}</h4>
                            <span>{location}</span>
                        </div>
                    }

                    if !ctx.props().event.teachers.is_empty() {
                        <div class="info-block">
                            <h4>{t("Enseignant")}</h4>
                            <span>{ctx.props().event.teachers.join(", ")}</span>
                        </div>
                    }
                </div>

                <div class="info-block">
                    <h4>{t("Couleur")}</h4>
                    <input type="color" id="background-color-input" value={bg_color.clone()} onchange={ctx.link().callback(|_| EventCompMsg::SaveColors)}  />

                    <span style={format!("background: {}80;", bg_color)}>{t("Fond")}</span>
                </div>
            </div>
        };

        html! {<>
            <div
                style={format!("background-color: {}80; border-left: 0.3rem solid {};  position: absolute; top: {}%; height: {}%;", bg_color.clone(), bg_color.clone(), percent_offset, percent_height)}
                class="event" >
                <div class="event-container"  onclick={if !self.popup_displayed{ Some(ctx.link().callback(|_| EventCompMsg::ShowPopup(true) ))} else { None } } >
                    <span class="name" >
                        { &name }
                    </span>
                    <span class="teacher">
                        { ctx.props().event.teachers.join(", ") }
                    </span>
                    if let Some(l) = &location { <span class="location" >{l}</span>}
                </div>
                if !mobile {{popup.clone()}} // Due to the brightness filter when events are pressed on mobile, the popup needs to be outside the event
            </div>
            if mobile {{popup}}
        </>}
    }
}
