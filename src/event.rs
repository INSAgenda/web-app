use agenda_parser::{Event, event::EventKind, location::Building};
use yew::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::TimeZone;
use chrono_tz::Europe::Paris;
use wasm_bindgen::{prelude::*, JsCast};
use crate::{settings::{SETTINGS, BuildingNaming}, colors::*};

lazy_static::lazy_static!{
    static ref ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Properties, Clone)]
pub struct EventCompProps {
    pub event: Event,
    pub day_start: u64,
    pub app_link: yew::html::Scope<crate::App>,
}

impl PartialEq for EventCompProps {
    fn eq(&self, other: &Self) -> bool {
        !COLORS_CHANGED.load(Ordering::Relaxed) && self.event.start_unixtime == other.event.start_unixtime && self.event.end_unixtime == other.event.end_unixtime // TODO: add other fields
    }
}

pub enum PopupPage {
    General,
    Colors,
}

pub struct EventComp {
    popup: Option<PopupPage>,
    on_click: Closure<dyn FnMut(web_sys::MouseEvent)>,
    ignore_next_event: bool,
    popup_id: String,
}

pub enum EventCompMsg {
    ToggleDetails,
    SetPage(PopupPage),
    SaveColors,
}

impl Component for EventComp {
    type Message = EventCompMsg;
    type Properties = EventCompProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = format!("event-popup-{}", ID_COUNTER.fetch_add(1, Ordering::Relaxed));

        // Creates a closure called on click that will close the popup if the user clicked outside of it
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let id2 = id.clone();
        let link = ctx.link().clone();
        let on_click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let popup_el = document.get_element_by_id(&id2).unwrap();
            let rect = popup_el.get_bounding_client_rect();

            // Check the click was not inside the popup
            if (rect.y()..rect.y()+rect.height()).contains(&(event.client_y() as f64))
                && (rect.x()..rect.x()+rect.width()).contains(&(event.client_x() as f64))
            { return; }

            link.send_message(EventCompMsg::ToggleDetails);
        }) as Box<dyn FnMut(_)>);

        EventComp {
            popup: None,
            on_click,
            ignore_next_event: false,
            popup_id: id,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EventCompMsg::ToggleDetails if self.popup.is_some() => {
                // We sometimes need a mechanism to ignore the next message. When the event listener is added, it instantly fires another ToggleDetails message which would close the popup instantly if not ignored
                if self.ignore_next_event {
                    self.ignore_next_event = false;
                    false
                } else {
                    self.popup = None;

                    // Popup is already closed, so we can spare resources by disabling the event listener
                    web_sys::window().unwrap().remove_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();
                    true
                }
            },
            EventCompMsg::ToggleDetails => {
                self.popup = Some(PopupPage::General);

                // Popup is now opened so we must be ready to close it
                web_sys::window().unwrap().add_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();

                self.ignore_next_event = true;

                true
            },
            EventCompMsg::SetPage(page) => {
                self.popup = Some(page);
                
                // We need this because instantly after this message will fire the click event but the popup content will already have changed and the click could be detected outside the popup
                self.ignore_next_event = true;

                true
            },
            EventCompMsg::SaveColors => {
                self.popup = Some(PopupPage::General);

                let document = web_sys::window().unwrap().document().unwrap();
                let background_color = match document.query_selector(&format!("#{} #background-color-input", self.popup_id)).unwrap() {
                    Some(el) => el.dyn_into::<web_sys::HtmlInputElement>().unwrap().value(),
                    None => return false,
                };
                let text_color = match document.query_selector(&format!("#{} #text-color-input", self.popup_id)).unwrap() {
                    Some(el) => el.dyn_into::<web_sys::HtmlInputElement>().unwrap().value(),
                    None => return false,
                };

                let kind = match &ctx.props().event.kind {
                    EventKind::Td(kind) => kind,
                    EventKind::Cm(kind) => kind,
                    EventKind::Tp(kind) => kind,
                    EventKind::Other(kind) => kind,
                };
                COLORS.set(kind, background_color, text_color);

                // We need to set this so that other events know that they have to refresh
                COLORS_CHANGED.store(true, Ordering::Relaxed);

                ctx.props().app_link.send_message(crate::Msg::Refresh);

                false
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sec_offset = ctx.props().event.start_unixtime.saturating_sub(ctx.props().day_start + 8 * 3600);
        let percent_offset = 100.0 / (44100.0) * sec_offset as f64;
        let percent_height = 100.0 / (44100.0) * (ctx.props().event.end_unixtime - ctx.props().event.start_unixtime) as f64;

        let (name, kind) = match &ctx.props().event.kind {
            EventKind::Td(kind) => (format!("TD: {}", kind), kind),
            EventKind::Cm(kind) => (format!("CM: {}", kind), kind),
            EventKind::Tp(kind) => (format!("TP: {}", kind), kind),
            EventKind::Other(kind) => (kind.to_string(), kind),
        };
        let (bg_color, text_color) = COLORS.get(kind);
        
        let location = ctx.props().event.location.map(|location| {
            let building =  match SETTINGS.building_naming() {
                BuildingNaming::Short => match location.building {
                    Building::Magellan => "Ma",
                    Building::DumontDurville => "Du",
                },
                BuildingNaming::Long => match location.building {
                    Building::Magellan => "Magellan",
                    Building::DumontDurville => "Dumont Durville",
                },
            };

            format!("{} - {} - {} - {}", building, location.building_area, location.level, location.room_number)
        });
    
        let start = Paris.timestamp(ctx.props().event.start_unixtime as i64, 0);
        let end = Paris.timestamp(ctx.props().event.end_unixtime as i64, 0);
        let duration = (ctx.props().event.end_unixtime - ctx.props().event.start_unixtime) / 60;
        let groups = ctx.props().event.groups.iter().map(|g| format!("{:?}", g)).collect::<Vec<_>>().join(", ");
        
        html! {
            <div
                style={format!("background-color: {}; color: {}; position: absolute; top: {}%; height: {}%;", bg_color, text_color, percent_offset, percent_height)}
                class="event"
                onclick={ if self.popup.is_none() { Some(ctx.link().callback(|_| EventCompMsg::ToggleDetails)) } else {None} } >

                <span class="name" >
                    { &name }
                </span>
                <span class="teacher">
                    { ctx.props().event.teachers.join(", ") }
                </span>
                {if let Some(l) = &location {html! {<span>{l}</span>}} else {html!{}}}
                <div class="event-details" id={self.popup_id.clone()} style={if self.popup.is_some() {""} else {"display: none;"}} >
                    <div class="event-details-header">
                        <span>{ name }</span>
                    </div>
                    { 
                        match self.popup {
                            Some(PopupPage::General) | None => html!{
                                <div class="event-details-content">
                                    <div>
                                        <span class="bold">{"Début : "}</span>
                                        {start.time().format("%Hh%M")}
                                    </div>
                                    <div>
                                        <span class="bold">{"Fin : "}</span>
                                        {end.time().format("%Hh%M")}
                                    </div>
                                    <div>
                                        <span class="bold">{"Durée : "}</span>
                                        {duration}{"min"}
                                    </div>
                                    <div>
                                        <span class="bold">{ if ctx.props().event.groups.len() > 1 {{"Groupes : "}} else {{"Groupe : "}} }</span>
                                        {groups}
                                    </div>
                                    <div>
                                        <span class="bold">{"Professeur : "}</span>
                                        {ctx.props().event.teachers.join(", ")}
                                    </div>
                                    <div>
                                        <span class="bold">{"Salle : "}</span>
                                        {location.unwrap_or_else(|| "Inconnue".to_string())}
                                    </div>
                                    <div class="bottom-buttons">
                                        <div onclick={ctx.link().callback(|_| EventCompMsg::SetPage(PopupPage::Colors))}>
                                            {"Changer les couleurs"}
                                            <img src="/agenda/images/edit-2.svg"/>
                                        </div>
                                    </div>
                                </div>
                            },
                            Some(PopupPage::Colors) => html!{
                                <div class="event-details-content color-editor-popup">
                                    <div>
                                        <span>{"Fond"}</span>
                                        <input type="color" id="background-color-input" value={bg_color} />
                                    </div>
                                    <div>
                                        <span>{"Texte"}</span>
                                        <input type="color" id="text-color-input" value={text_color} />
                                    </div>
                                    <div class="bottom-buttons">
                                        <div onclick={ctx.link().callback(|_| EventCompMsg::SetPage(PopupPage::General))}>
                                            {"Annuler"}
                                            <img src="/agenda/images/x.svg"/>
                                        </div>
                                        <div onclick={ctx.link().callback(|_| EventCompMsg::SaveColors)}>
                                            {"Sauvegarder"}
                                            <img src="/agenda/images/save.svg"/>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    }
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        let mut element = if let Some(el) = web_sys::window().unwrap().document().unwrap().get_element_by_id( &self.popup_id){el} else {return};
        let mut count = 0;
        loop{
            element = if let Some(el) = element.previous_element_sibling(){el} else {break};
            count += element.inner_html().len();
        }
        let parent = element.parent_element().unwrap();
        let cur_style = parent.get_attribute("style").unwrap();
        let size = 6.0 * ((parent.client_width().pow(2) as f64 + parent.client_height().pow(2) as f64).sqrt()/count as f64);

        parent.set_attribute("style", &format!("{} font-size: min({}px, 1rem);", cur_style, size)).unwrap();
    }
    
}
