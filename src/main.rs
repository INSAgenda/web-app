//! Welcome to the INSAgenda web app! This is the main file of the app.
//! INSAgenda uses the [Yew](https://yew.rs) framework to build a single page web app.
//! In this file, we define the main component of the app, which is the `App` component.
//! The `App` component is charged of managing the page that is currently displayed, and stores data that is shared between pages.

#[path = "alert/alert.rs"]
mod alert;
#[path = "announcement/announcement.rs"]
mod announcement;
#[path = "event/event.rs"]
mod event;
#[path = "settings/settings.rs"]
mod settings;
#[path = "agenda/agenda.rs"]
mod agenda;
#[path = "glider_selector/glider_selector.rs"]
mod glider_selector;
#[path = "calendar/calendar.rs"]
mod calendar;
#[path = "crash/crash_handler.rs"]
mod crash_handler;
#[path = "change_data/change_data.rs"]
mod change_data;
#[path ="popup/popup.rs"]
mod popup;
#[path = "survey/survey.rs"]
mod survey;
#[path = "checkbox/checkbox.rs"]
mod checkbox;
#[path = "sortable/sortable.rs"]
mod sortable;
mod util;
mod slider;
mod api;
mod prelude;
mod translation;
mod colors;

use slider::width;

use crate::{prelude::*, settings::SettingsPage, change_data::ChangeDataPage};

/// The page that is currently displayed.
pub enum Page {
    Settings,
    ChangePassword,
    ChangeEmail,
    ChangeGroup,
    Agenda,
    Popup(PopupState),
    Survey { sid: String },
}

/// A message that can be sent to the `App` component.
pub enum Msg {
    /// Switch page
    SetPage(Page),
    /// Switch page without saving it in the history
    SilentSetPage(Page),
    FetchColors(HashMap<String, String>),
    SaveSurveyAnswer(SurveyAnswers),

    // Data updating messages sent by the loader in /src/api/generic.rs
    UserInfoSuccess(UserInfo),
    GroupsSuccess(Vec<GroupDesc>),
    ApiFailure(ApiError),
    ScheduleSuccess(Vec<RawEvent>),
    SurveysSuccess(Vec<Survey>, Vec<SurveyAnswers>),
    ScheduleFailure(ApiError),
    AnnouncementsSuccess(Vec<AnnouncementDesc>),
}

/// The main component of the app.
/// Stores data that is shared between pages, as well as the page that is currently displayed.
pub struct App {
    groups: Rc<Vec<GroupDesc>>,
    user_info: Rc<Option<UserInfo>>,
    events: Rc<Vec<RawEvent>>,
    announcements: Rc<Vec<AnnouncementDesc>>,
    surveys: Vec<Survey>,
    survey_answers: Vec<SurveyAnswers>,
    page: Page,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        crash_handler::init();

        // Handle popstate events (back browser button)
        let link2 = ctx.link().clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::PopStateEvent| {
            let state = e.state().as_string();
            match state.as_deref() {
                Some("settings") => link2.send_message(Msg::SilentSetPage(Page::Settings)),
                Some("agenda") => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                Some("change-password") => link2.send_message(Msg::SilentSetPage(Page::ChangePassword)),
                Some("change-email") => link2.send_message(Msg::SilentSetPage(Page::ChangeEmail)),
                Some("change-group") => link2.send_message(Msg::SilentSetPage(Page::ChangeGroup)),
                Some(survey) if survey.starts_with("survey/") => link2.send_message(Msg::SilentSetPage(Page::Survey { sid: survey[7..].to_string() })),
                _ if e.state().is_null() => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                _ => alert(format!("Unknown pop state: {:?}", e.state())),
            }
        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();

        // Update data
        let events = CachedData::init(ctx.link().clone()).unwrap_or_default();
        let user_info = CachedData::init(ctx.link().clone());
        let mut announcements: Vec<AnnouncementDesc> = CachedData::init(ctx.link().clone()).unwrap_or_default();
        let groups = CachedData::init(ctx.link().clone()).unwrap_or_default();
        let survey_response: SurveyResponse = CachedData::init(ctx.link().clone()).unwrap_or_default();
        let surveys = survey_response.surveys;
        let survey_answers = survey_response.my_answers;
        if window().navigator().on_line() { // temporary
            announcements.append(&mut surveys_to_announcements(&surveys, &survey_answers));
        }

        // Open corresponding page
        let page = match window().location().pathname().unwrap_or_default().as_str() {
            "/settings" => Page::Settings,
            "/change-password" => Page::ChangePassword,
            "/change-email" => Page::ChangeEmail,
            "/change-group" => Page::ChangeGroup,
            survey if survey.starts_with("/survey/") => Page::Survey { sid: survey[8..].to_string() },
            "/agenda" => match window().location().hash() { // For compatibility with old links
                Ok(hash) if hash == "#settings" => Page::Settings,
                Ok(hash) if hash == "#change-password" => Page::ChangePassword,
                Ok(hash) if hash == "#change-email" => Page::ChangeEmail,
                Ok(hash) if hash == "#change-group" => Page::ChangeGroup,
                Ok(hash) if hash.starts_with("#survey-") => Page::Survey { sid: hash[8..].to_string() },
                Ok(hash) if hash.is_empty() => Page::Agenda,
                Ok(hash) => {
                    alert(format!("Page {hash} not found"));
                    Page::Agenda
                },
                _ => Page::Agenda,
            }
            pathname => {
                alert(format!("Page {pathname} not found"));
                Page::Agenda
            }
        };

        // Open survey if one is available and required
        if window().navigator().on_line() { // temporary
            let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;
            if let Some(survey_to_open) = surveys.iter().find(|s| s.required && s.start_ts <= now && s.end_ts >= now) {
                if !survey_answers.iter().any(|a| a.id == survey_to_open.id) {
                    ctx.link().send_message(Msg::SetPage(Page::Survey { sid: survey_to_open.id.clone() }));
                }
            }
        }

        Self {
            events: Rc::new(events),
            user_info: Rc::new(user_info),
            announcements: Rc::new(announcements),
            groups: Rc::new(groups),
            surveys,
            survey_answers,
            page,
        }
    }

    /// Most of the messages handled in the function are sent by the data loader to update the data or report an error.
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::AnnouncementsSuccess(announcements) => {
                self.announcements = Rc::new(announcements);
                false // Don't think we should refresh display of the page because it would cause high inconvenience and frustration to the users
            },
            AppMsg::ScheduleSuccess(events) => {
                self.events = Rc::new(events);
                true
            },
            AppMsg::SurveysSuccess(surveys, survey_answers) => {
                self.surveys = surveys;
                self.survey_answers = survey_answers;
                if !self.surveys.is_empty() {
                    // TODO sort surveys by required
                    if !self.survey_answers.iter().any(|a| a.id == self.surveys[0].id) {
                        ctx.link().send_message(Msg::SetPage(Page::Survey { sid: self.surveys[0].id.clone() }));
                    }
                }
                false
            },
            AppMsg::SaveSurveyAnswer(answers) => {
                self.survey_answers.retain(|s| s.id != answers.id);
                self.survey_answers.push(answers);
                let to_save = SurveyResponse {
                    surveys: self.surveys.clone(),
                    my_answers: self.survey_answers.clone(),
                };
                to_save.save();
                false
            }
            Msg::UserInfoSuccess(user_info) => {
                let mut should_refresh = false;

                if let Some(old_user_info) = self.user_info.as_ref() {
                    if old_user_info.user_groups != user_info.user_groups {
                        self.events = Rc::new(Vec::new());
                        <Vec<RawEvent>>::refresh(ctx.link().clone());
                        should_refresh = true;
                    }
                }

                // Update user info
                user_info.save();
                self.user_info = Rc::new(Some(user_info));

                should_refresh
            },
            Msg::GroupsSuccess(groups) => {
                self.groups = Rc::new(groups);
                false
            },
            AppMsg::ScheduleFailure(api_error) => {
                api_error.handle_api_error();
                if self.events.is_empty() {
                    alert("Failed to fetch schedule");
                }
                false
            },
            Msg::ApiFailure(api_error) => {
                api_error.handle_api_error();
                false
            },
            Msg::SetPage(mut page) => {
                let history = window().history().expect("Failed to access history");
                if let Page::Popup(PopupState::Opened { popup_size, .. }) = &mut page {
                    //self.slider.borrow_mut().disable();
                    if let Page::Popup(PopupState::Opened { popup_size: Some(previous_size), .. }) | Page::Popup(PopupState::Closing { popup_size: Some(previous_size), .. }) = self.page {
                        *popup_size = Some(previous_size);
                    } else if let Some(day_el) = window().doc().get_element_by_id("day0") {
                        let rect = day_el.get_bounding_client_rect();
                        *popup_size = Some((width() as f64 - rect.width() - 2.0 * rect.left()) as usize)
                    }
                    spawn_local(async move {
                        window().doc().body().unwrap().set_attribute("style", "overflow: hidden").unwrap();
                        sleep(Duration::from_millis(500)).await;
                        window().doc().body().unwrap().remove_attribute("style").unwrap();
                    });
                }
                if let Page::Popup(PopupState::Closing { popup_size, .. } ) = &mut page {
                    if let Page::Popup(PopupState::Opened { popup_size: Some(previous_size), .. }) | Page::Popup(PopupState::Closing { popup_size: Some(previous_size), .. }) = self.page {
                        *popup_size = Some(previous_size);
                    } else if let Some(day_el) = window().doc().get_element_by_id("day0") {
                        let rect = day_el.get_bounding_client_rect();
                        *popup_size = Some((width() as f64 - rect.width() - 2.0 * rect.left()) as usize)
                    }
                    let link = ctx.link().clone();
                    spawn_local(async move {
                        window().doc().body().unwrap().set_attribute("style", "overflow: hidden").unwrap();
                        sleep(Duration::from_millis(500)).await;
                        link.send_message(Msg::SetPage(Page::Popup(PopupState::Closed)));
                        window().doc().body().unwrap().remove_attribute("style").unwrap();
                    });
                }
                if let Page::Popup(PopupState::Closed) = &page {
                    //self.slider.borrow_mut().enable();
                    page = Page::Agenda;
                }
                match &page {
                    Page::Settings => history.push_state_with_url(&JsValue::from_str("settings"), "Settings", Some("/settings")).unwrap(),
                    Page::ChangePassword => history.push_state_with_url(&JsValue::from_str("change-password"), "Change password", Some("/change-password")).unwrap(),
                    Page::ChangeEmail => history.push_state_with_url(&JsValue::from_str("change-email"), "Change email", Some("/change-email")).unwrap(),
                    Page::ChangeGroup => history.push_state_with_url(&JsValue::from_str("change-group"), "Change group", Some("/change-group")).unwrap(),
                    Page::Agenda => history.push_state_with_url(&JsValue::from_str("agenda"), "Agenda", Some("/agenda")).unwrap(),
                    Page::Survey { sid } => history.push_state_with_url(&JsValue::from_str(&format!("survey/{sid}")), "Survey", Some(&format!("/survey/{sid}"))).unwrap(),
                    Page::Popup(_) => history.push_state_with_url(&JsValue::from_str("popup"), "Popup", Some("/popup")).unwrap(),
                }
                self.page = page;
                true
            },
            Msg::SilentSetPage(page) => {
                self.page = page;
                true
            },
            Msg::FetchColors(new_colors) => {
                crate::COLORS.update_colors(new_colors);
                true
            },
        }
    }
    
    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        crate::colors::COLORS_CHANGED.store(false, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.page {
            Page::Agenda => {
                let user_info = Rc::clone(&self.user_info);
                let events = Rc::clone(&self.events);
                let announcements = Rc::clone(&self.announcements);
                html!(<Agenda events={events} user_info={user_info} announcements={announcements} app_link={ctx.link().clone()} popup={PopupState::Closed} />)
            },
            Page::Popup(popup_state) => {
                let user_info = Rc::clone(&self.user_info);
                let events = Rc::clone(&self.events);
                let announcements = Rc::clone(&self.announcements);
                html!(<Agenda events={events} user_info={user_info} announcements={announcements} app_link={ctx.link().clone()} popup={popup_state.clone()} />)
            },
            Page::Settings => html!( <SettingsPage app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
            Page::ChangePassword => html!(
                <ChangeDataPage
                    kind="new_password"
                    app_link={ ctx.link().clone() }
                    user_info={Rc::clone(&self.user_info)}
                    groups={Rc::clone(&self.groups)} />
            ),
            Page::ChangeEmail => html!(
                <ChangeDataPage
                    kind="email"
                    app_link={ ctx.link().clone() }
                    user_info={Rc::clone(&self.user_info)}
                    groups={Rc::clone(&self.groups)} />
            ),
            Page::ChangeGroup => html!(
                <ChangeDataPage
                    kind="group"
                    app_link={ ctx.link().clone() }
                    user_info={Rc::clone(&self.user_info)}
                    groups={Rc::clone(&self.groups)} />
            ),
            Page::Survey { sid } => {
                let survey = match self.surveys.iter().find(|s| s.id == *sid) {
                    Some(s) => s,
                    None => {
                        redirect("/agenda");
                        return html!();
                    }
                };
                let answers = self.survey_answers.iter().find(|s| s.id == *sid).map(|a| a.answers.to_owned());
                html!(<SurveyComp survey={survey.clone()} answers={answers} app_link={ctx.link().clone()} />)
            },
        }
    }
}

/// Redirect the user
fn redirect(page: &str) {
    let _ = window().location().set_href(page);
    log!("Redirecting to {page}");
}

/// Set status to running
fn confirm_running(window: &web_sys::Window) {
    let local_storage = window.local_storage().unwrap().unwrap();
    local_storage.set_item("wasm-running-status", "running").unwrap();
}

/// Prevent webdrivers from accessing the page
fn stop_bots(window: &web_sys::Window) {
    if js_sys::Reflect::get(&window.navigator(), &"webdriver".to_string().into()).unwrap().as_bool().unwrap_or(false) {
        panic!("Your browser failed load this page");
    }
}

fn main() {
    let window = web_sys::window().expect("Please run the program in a browser context");
    confirm_running(&window);
    stop_bots(&window);
    let doc = window.doc();
    let element = doc.get_element_by_id("render").unwrap();
    yew::Renderer::<App>::with_root(element).render();
}
