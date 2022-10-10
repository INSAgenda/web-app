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
        let on_click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let popup_el = doc.get_element_by_id("event-details").unwrap();
            let rect = popup_el.get_bounding_client_rect();

            // Check the click was not inside the popup
            if (rect.y()..rect.y()+rect.height()).contains(&(event.client_y() as f64))
                && (rect.x()..rect.x()+rect.width()).contains(&(event.client_x() as f64))
            { return; }
            link.send_message(PopupMsg::Close);

        }) as Box<dyn FnMut(_)>);

        Self {
            on_click,
            last_click_timestamp: 0,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if ctx.props().event.is_some() {
            self.last_click_timestamp = chrono::Utc::now().timestamp_millis();
            window().add_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();
        } else if ctx.props().event.is_none() {
            window().remove_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();
        }
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
                // Double call protection
                let now = chrono::Utc::now().timestamp_millis();
                if self.last_click_timestamp + 100 > now {
                    return false;
                }
                self.last_click_timestamp = now;

                // Enable/Disable slider 
                ctx.props().agenda_link.send_message(crate::AgendaMsg::SetSelectedEvent(None));
                true
            },
        }
    }
  
    fn view(&self, ctx: &Context<Self>) -> Html {
        // Format title
        let binding = &ctx.props().event;
        let event = match binding.as_ref() {
            Some(e) => e,
            None => return html! {<div></div>},
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

        template_html!(
            "templates/components/popup.html",
            onclick_close = {ctx.link().callback(move |_| PopupMsg::Close)},
            onclick_save = {ctx.link().callback(move |_| PopupMsg::SaveColors)},
            time = {format!("{} - {}", start.time().format("%Hh%M"), end.time().format("%Hh%M"))},
            teachers = {event.teachers.join(", ")},
            teachers_empty = {event.teachers.is_empty()},
            mobile, name, bg_color = { bg_color.clone() }, opt_location = {location}
        )
    }
}