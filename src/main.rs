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
mod util;
#[path = "calendar/calendar.rs"]
mod calendar;
mod slider;
mod api;
#[path = "crash/crash_handler.rs"]
mod crash_handler;
mod colors;
#[path = "change_data/change_data.rs"]
mod change_data;
mod prelude;
mod translation;
#[path ="popup/popup.rs"]
mod popup;
#[path = "survey/survey.rs"]
mod survey;
#[path = "checkbox/checkbox.rs"]
mod checkbox;
#[path = "sortable/sortable.rs"]
mod sortable;

use crate::{prelude::*, settings::SettingsPage, change_data::ChangeDataPage};

pub enum Page {
    Settings,
    ChangePassword,
    ChangeEmail,
    ChangeGroup,
    Agenda,
    Survey(Rc<Survey>),
}

pub enum Msg {
    UserInfoSuccess(UserInfo),
    GroupsSuccess(Vec<GroupDesc>),
    ApiFailure(ApiError),
    SetPage(Page),
    SilentSetPage(Page),
    ScheduleSuccess(Vec<RawEvent>),
    SurveysSuccess(Vec<Survey>, Vec<SurveyAnswers>),
    ScheduleFailure(ApiError),
    AnnouncementsSuccess(Vec<AnnouncementDesc>),
    FetchColors(HashMap<String, String>),
}

pub struct App {
    groups: Rc<Vec<GroupDesc>>,
    user_info: Rc<Option<UserInfo>>,
    events: Rc<Vec<RawEvent>>,
    announcements: Rc<Vec<AnnouncementDesc>>,
    surveys: Vec<Survey>,
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
        announcements.append(&mut surveys_to_announcements(&surveys, &survey_answers));

        // Detect page
        let page = match window().location().hash() {
            Ok(hash) if hash == "#settings" => Page::Settings,
            Ok(hash) if hash == "#change-password" => Page::ChangePassword,
            Ok(hash) if hash == "#change-email" => Page::ChangeEmail,
            Ok(hash) if hash == "#change-group" => Page::ChangeGroup,
            Ok(hash) if hash.starts_with("#survey-") => {
                let id = hash[8..].to_string();
                surveys.iter().find(|s| s.id == id).map(|s| Page::Survey(Rc::new(s.clone()))).unwrap_or(Page::Agenda)
            }
            Ok(hash) if hash.is_empty() => Page::Agenda,
            Ok(hash) => {
                alert(format!("Page {hash} not found"));
                Page::Agenda
            },
            _ => Page::Agenda,
        };

        // Open survey if one is available and required
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;
        if let Some(survey_to_open) = surveys.iter().find(|s| s.required && s.start_ts <= now && s.end_ts >= now) {
            ctx.link().send_message(Msg::SetPage(Page::Survey(Rc::new(survey_to_open.clone()))));
        }

        Self {
            events: Rc::new(events),
            user_info: Rc::new(user_info),
            announcements: Rc::new(announcements),
            groups: Rc::new(groups),
            surveys,
            page,
        }
    }

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
                if !self.surveys.is_empty() {
                    ctx.link().send_message(AppMsg::SetPage(Page::Survey(Rc::new(self.surveys[0].clone()))));
                }
                true
            },
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
                let history = window().history().expect("Failed to access history");                
                match &page {
                    Page::Settings => history.push_state_with_url(&JsValue::from_str("settings"), "Settings", Some("#settings")).unwrap(),
                    Page::Agenda => history.push_state_with_url(&JsValue::from_str("agenda"), "Agenda", Some("/agenda")).unwrap(),
                    Page::Survey(survey) => history.push_state_with_url(&JsValue::from_str("survey"), "Survey", Some(&format!("#survey-{}", survey.id))).unwrap(),
                    Page::ChangePassword => history.push_state_with_url(&JsValue::from_str("change-password"), "Change password", Some("#change-password")).unwrap(),
                    Page::ChangeEmail => history.push_state_with_url(&JsValue::from_str("change-email"), "Change email", Some("#change-email")).unwrap(),
                    Page::ChangeGroup => history.push_state_with_url(&JsValue::from_str("change-group"), "Change group", Some("#change-group")).unwrap(),
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
                html!(<Agenda events={events} user_info={user_info} announcements={announcements} app_link={ctx.link().clone()} />)
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
            Page::Survey(survey) => html!(<SurveyComp survey={survey} app_link={ctx.link().clone()} />),
        }
    }
}

/// Redirect the user
fn redirect(page: &str){
    let _ = window().location().set_href(page);
    log!("Redirecting to {page}");
}

/// Prevent webdrivers from accessing the page
fn stop_bots(window: &web_sys::Window) {
    if js_sys::Reflect::get(&window.navigator(), &"webdriver".to_string().into()).unwrap().as_bool().unwrap_or(false) {
        panic!("Your browser failed load this page");
    }
}

fn main() {
    let window = web_sys::window().expect("Please run the program in a browser context");
    stop_bots(&window);
    let doc = window.doc();
    let element = doc.get_element_by_id("render").unwrap();
    yew::start_app_in_element::<App>(element);
}
