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

use crate::{prelude::*, settings::SettingsPage, change_data::ChangeDataPage};

/// The page that is currently displayed.
#[derive(Clone, Routable, PartialEq)]
pub enum Page {
    #[at("/settings")]
    Settings,
    #[at("/change-password")]
    ChangePassword,
    #[at("/change-email")]
    ChangeEmail,
    #[at("/change-group")]
    ChangeGroup,
    #[at("/agenda")]
    Agenda,
    #[at("/survey/:sid")]
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
    surveys: Rc<Vec<Survey>>,
    survey_answers: Rc<Vec<SurveyAnswers>>,
    page: Page,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        crash_handler::init();

        // Redirections
        let link2 = ctx.link().clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::PopStateEvent| {
            let state = e.state().as_string();
            match state.as_deref() {
                Some("settings") => link2.send_message(Msg::SilentSetPage(Page::Settings)),
                Some("agenda") => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                Some("change-password") => link2.send_message(Msg::SilentSetPage(Page::ChangePassword)),
                Some("change-email") => link2.send_message(Msg::SilentSetPage(Page::ChangeEmail)),
                Some("change-group") => link2.send_message(Msg::SilentSetPage(Page::ChangeGroup)),
                // TODO survey
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

        // Detect page
        let page = Page::Agenda;

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
            surveys: Rc::new(surveys),
            survey_answers: Rc::new(survey_answers),
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
                self.surveys = Rc::new(surveys);
                self.survey_answers = Rc::new(survey_answers);
                if !self.surveys.is_empty() {
                    // TODO sort surveys by required
                    if !self.survey_answers.iter().any(|a| a.id == self.surveys[0].id) {
                        ctx.link().send_message(Msg::SetPage(Page::Survey {sid: self.surveys[0].id.clone() }));
                    }
                }
                false
            },
            AppMsg::SaveSurveyAnswer(answers) => {
                let mut survey_answers = self.survey_answers.deref().to_owned();
                survey_answers.retain(|s| s.id != answers.id);
                survey_answers.push(answers);
                let to_save = SurveyResponse {
                    surveys: self.surveys.deref().to_owned(),
                    my_answers: survey_answers.clone(),
                };
                self.survey_answers = Rc::new(survey_answers);
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
            Msg::SetPage(page) => {
                if let Some(navigator) = ctx.link().navigator() {
                    navigator.push(&page);
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
        let user_info = Rc::clone(&self.user_info);
        let events = Rc::clone(&self.events);
        let announcements = Rc::clone(&self.announcements);
        let groups = Rc::clone(&self.groups);
        let surveys = Rc::clone(&self.surveys);
        let survey_answers = Rc::clone(&self.survey_answers);
        let app_link = ctx.link().clone();

        let switch = move |page| match page {
            Page::Agenda => {
                let events = Rc::clone(&events);
                let announcements = Rc::clone(&announcements);
                let user_info = Rc::clone(&user_info);
                html!(<Agenda events={events} user_info={user_info} announcements={announcements} app_link={app_link.clone()} />)
            },
            Page::Settings => {
                let user_info = Rc::clone(&user_info);
                html!( <SettingsPage app_link={app_link.clone()} user_info={user_info} /> )
            },
            Page::ChangePassword => {
                let user_info = Rc::clone(&user_info);
                let groups = Rc::clone(&groups);
                html!(
                    <ChangeDataPage
                        kind="new_password"
                        app_link={ app_link.clone() }
                        user_info={user_info}
                        groups={groups} />
                )
            },
            Page::ChangeEmail => {
                let user_info = Rc::clone(&user_info);
                let groups = Rc::clone(&groups);
                html!(
                    <ChangeDataPage
                        kind="email"
                        app_link={ app_link.clone() }
                        user_info={user_info}
                        groups={groups} />
                )
            },
            Page::ChangeGroup => {
                let user_info = Rc::clone(&user_info);
                let groups = Rc::clone(&groups);
                html!(
                    <ChangeDataPage
                        kind="group"
                        app_link={ app_link.clone() }
                        user_info={user_info}
                        groups={groups} />
                )
            },
            Page::Survey { sid } => {
                let survey = surveys.iter().find(|s| s.id == *sid).unwrap(); // TODO unwrap
                let answers = survey_answers.iter().find(|a| a.id == *sid).map(|s| s.answers.to_owned());
                html!(<SurveyComp survey={survey.to_owned()} answers={answers} app_link={app_link.clone()} />)
            },
        };

        html! {
            <BrowserRouter>
                <Switch<Page> render={switch} />
            </BrowserRouter>
        }
    }
}

/// Redirect the user
fn redirect(page: &str){
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
