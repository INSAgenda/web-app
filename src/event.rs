
use crate::{prelude::*, slider::width};

lazy_static::lazy_static!{
    static ref ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Properties, Clone)]
pub struct EventCompProps {
    pub event: RawEvent,
    pub day_start: u64,
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
    ignore_next_event: bool,
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
        let document = window().document().unwrap();
        let id2 = id.clone();
        let link = ctx.link().clone();
        let on_click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let popup_el = document.get_element_by_id(&id2).unwrap();
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
            ignore_next_event: false,
            popup_id: id,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {

            EventCompMsg::ShowPopup(state) => {
                if self.ignore_next_event{
                    self.ignore_next_event = false;
                    return false;
                }

                self.popup_displayed = state;
                if self.popup_displayed {
                    self.ignore_next_event = true;
                    window().add_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();
                    if width() <= 1000 {
                        ctx.props().app_link.send_message(AppMsg::SliderState(false));
                    }
                    
                } else {
                    window().remove_event_listener_with_callback("click", self.on_click.as_ref().unchecked_ref()).unwrap();
                    if width() <= 1000 {
                        ctx.props().app_link.send_message(AppMsg::SliderState(true));
                    }
                                    }
                true
            },
            EventCompMsg::SaveColors => {
                let document = window().document().unwrap();
                let background_color = match document.query_selector(&format!("#{} #background-color-input", self.popup_id)).unwrap() {
                    Some(el) => el.dyn_into::<HtmlInputElement>().unwrap().value(),
                    None => return false,
                };
                let text_color = String::new();

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
        let (bg_color, _) = COLORS.get(kind);
        
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
        let class = String::new() + if self.popup_displayed {""} else {"hide"};
        html! {
            <div
                style={format!("background-color: {}80; border-left: 0.3rem solid {};  position: absolute; top: {}%; height: {}%;", bg_color.clone(), bg_color.clone(), percent_offset, percent_height)}
                class="event"
                onclick={ Some(ctx.link().callback(|_| EventCompMsg::ShowPopup(true))) }  >
                <span class="name" >
                    { &name }
                </span>
                <span class="teacher">
                    { ctx.props().event.teachers.join(", ") }
                </span>
                {if let Some(l) = &location {html! {<span class="location" >{l}</span>}} else {html!{}}}
                <div class={format!("event-details {}", class)} id={self.popup_id.clone()} style={String::new() + if ctx.props().day_of_week > 2 { "left" } else { "right" } + ": -214px;"} >
                        <h3>{ name }</h3>
                        <div style={format!("background-color: {};", bg_color.clone())} class="divider-bar-option"></div>                   
                        <div  class="event-details-content">
                            <div class="info-block">
                                <h4>{t("Horaires")}</h4>
                                <span>{format!("{} - {}",
                                 start.time().format("%Hh%M"), end.time().format("%Hh%M"))}</span>
                            </div>
                            if location.is_some() {
                            <div class="info-block">
                                <h4>{t("Emplacement")}</h4>
                                <span>{location.unwrap_or_else(|| t("Inconnu").to_string())}</span>
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
                </div>
        }
    }
}
