use crate::{prelude::*, slider::width};

pub struct Popup {
    on_click: Closure<dyn FnMut(web_sys::MouseEvent)>,
    last_click_timestamp: i64,
}

pub enum PopupMsg {
    Close,
    SaveColors,
}

#[derive(Properties, Clone)]
pub struct PopupProps {
    pub event: Rc<Option<RawEvent>>,
    pub agenda_link: Scope<Agenda>,
}

impl PartialEq for PopupProps {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event
    }
}

impl Component for Popup {
    type Message = PopupMsg;
    type Properties = PopupProps;

    fn create(ctx: &Context<Self>) -> Self {
        // Creates a closure called on click that will close the popup if the user clicked outside of it
        let doc = window().doc();
        let link = ctx.link().clone();
        let agenda_link = ctx.props().agenda_link.clone();
        let on_click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let popup_el = doc.get_element_by_id("event-details").unwrap();
            let rect = popup_el.get_bounding_client_rect();

            // Check the click was not inside the popup
            if (rect.y()..rect.y()+rect.height()).contains(&(event.client_y() as f64))
                && (rect.x()..rect.x()+rect.width()).contains(&(event.client_x() as f64))
            { return; }
            agenda_link.send_message(crate::AgendaMsg::SetSelectedEvent(None));

        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("onclick", on_click.as_ref().unchecked_ref()).unwrap();

        Self {
            on_click,
            last_click_timestamp: 0,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log!("Event selected: {:?}", ctx.props().event);
        match msg {
            PopupMsg::SaveColors => {
                if let Some(event) = &ctx.props().event.as_ref() {
                    let document = window().doc();
                    let background_color = match document.query_selector("#background-color-input").unwrap() {
                        Some(el) => el.dyn_into::<HtmlInputElement>().unwrap().value(),
                        None => return false,
                    };

                    COLORS.set(&event.summary, background_color); 

                    // We need to set this so that other events know that they have to refresh
                    COLORS_CHANGED.store(true, Ordering::Relaxed);

                    ctx.props().agenda_link.send_message(crate::AgendaMsg::Refresh);
                }
                false
            },
            PopupMsg::Close => {
                //ctx.props().agenda_link.send_message(crate::AgendaMsg::SetSelectedEvent(None));
                // Double call protection
                /*let now = chrono::Utc::now().timestamp_millis();
                if self.last_click_timestamp + 100 > now {
                    return false;
                }
                self.last_click_timestamp = now;*/
                /*{
                    let mut event: RefMut<Option<common::Event>> = ctx.props().event.borrow_mut();
                    *event = None;
    
                }
                log!("Closing popup: {:?}" , ctx.props().event.borrow());
                // Enable/Disable slider 
                if width() <= 1000 {
                    ctx.props().agenda_link.send_message(AgendaMsg::SetSliderState(false));
                }*/
                true
            },
        }
    }
  
    fn view(&self, ctx: &Context<Self>) -> Html {
        // Format title
        let binding = &ctx.props().event;
        let event = match binding.as_ref() {
            Some(e) => {log!("Event: {:?}", e); e},
            None => {
                log!("No event");
                ctx.props().agenda_link.send_message(AgendaMsg::SetSliderState(true));
                return html! {<div></div>}
            },
        }; 

        let summary = &event.summary;
        let name = match event.kind {
            Some(EventKind::Td) => format!("TD: {summary}"),
            Some(EventKind::Tp) => format!("TP: {summary}"),
            Some(EventKind::Cm) => format!("CM: {summary}"),
            None => summary.clone(),
        };
        let bg_color = COLORS.get(&event.summary);

        // Format location
        let location = event.location.as_ref().map(|location| {
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

        let start = Paris.timestamp(event.start_unixtime as i64, 0);
        let end = Paris.timestamp(event.end_unixtime as i64, 0);
        let mobile = width() <= 1000;

        html! {
            <div id={"event-details"}>
                if mobile {
                    <div class="close-arrow" onclick={ctx.props().agenda_link.callback(|_| crate::AgendaMsg::SetSelectedEvent(None))} >
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
                <div id="event-details-content">
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

                    if !event.teachers.is_empty() {
                        <div class="info-block">
                            <h4>{t("Enseignant")}</h4>
                            <span>{event.teachers.join(", ")}</span>
                        </div>
                    }
                </div>

                <div class="info-block">
                    <h4>{t("Couleur")}</h4>
                    <input type="color" id="background-color-input" value={bg_color.clone()} onchange={ctx.link().callback(|_| PopupMsg::SaveColors)}  />

                    <span style={format!("background: {}80;", bg_color)}>{t("Fond")}</span>
                </div>
            </div>
        }
    }
}