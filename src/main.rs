//! Welcome to the INSAgenda web app! This is the main file of the app.
//! INSAgenda uses the [Yew](https://yew.rs) framework to build a single page web app.
//! In this file, we define the main component of the app, which is the `App` component.
//! The `App` component is charged of managing the page that is currently displayed, and stores data that is shared between pages.

#[path = "alert/alert.rs"]
mod alert;
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
#[path ="popup/popup.rs"]
mod popup;
#[path = "survey/survey.rs"]
mod survey;
#[path = "checkbox/checkbox.rs"]
mod checkbox;
#[path = "sortable/sortable.rs"]
mod sortable;
#[path = "tabbar/tabbar.rs"]
mod tabbar;
#[path = "friends/friends.rs"]
mod friends;
#[path = "comment/comment.rs"]
mod comment;
#[path = "notifications/notifications.rs"]
mod notifications;
mod util;
mod slider;
mod api;
mod prelude;
mod translation;
mod colors;

use slider::width;
use yew::virtual_dom::VNode;

use crate::{prelude::*, settings::SettingsPage};

/// The page that is currently displayed.
#[derive(Clone, PartialEq)]
pub enum Page {
    Settings,
    Agenda,
    Friends,
    FriendAgenda { pseudo: String },
    Notifications,
    Event { eid: String },
    Survey { sid: String },
    Rick,
}

impl Page {
    fn data_and_title(&self) -> (String, &'static str) {
        match self {
            Page::Settings => (String::from("settings"), "Settings"),
            Page::Agenda => (String::from("agenda"), "Agenda"),
            Page::Friends => (String::from("friends"), "Friends"),
            Page::FriendAgenda { pseudo } => (format!("friend-agenda/{pseudo}"), "Friend agenda"),
            Page::Notifications => (String::from("notifications"), "Notifications"),
            Page::Survey { sid } => (format!("survey/{sid}"), "Survey"),
            Page::Event { eid } => (format!("event/{eid}"), "Event"),
            Page::Rick => (String::from("r"), "Rick"),
        }
    }
}

/// A message that can be sent to the `App` component.
pub enum Msg {
    /// Switch page
    SetPage(Page),
    /// Switch page without saving it in the history
    SilentSetPage(Page),
    FetchColors(HashMap<String, String>),
    SaveSurveyAnswer(SurveyAnswers),
    UpdateFriends(FriendLists),
    MarkCommentsAsSeen(String),

    // Data updating messages sent by the loader in /src/api/generic.rs
    UserInfoSuccess(UserInfo),
    FriendsSuccess(FriendLists),
    FriendsEventsSuccess{ uid: i64, events: Vec<RawEvent> },
    CommentCountsSuccess(CommentCounts),
    ApiFailure(ApiError),
    ScheduleSuccess(Vec<RawEvent>),
    SurveysSuccess(Vec<Survey>, Vec<SurveyAnswers>),
    ScheduleFailure(ApiError),
    AnnouncementsSuccess(Vec<AnnouncementDesc>),
}

/// The main component of the app.
/// Stores data that is shared between pages, as well as the page that is currently displayed.
pub struct App {
    user_info: Rc<Option<UserInfo>>,
    events: Rc<Vec<RawEvent>>,
    announcements: Rc<Vec<AnnouncementDesc>>,
    notifications: Rc<RefCell<LocalNotificationTracker>>,
    friends: Rc<Option<FriendLists>>,
    friends_events: FriendsEvents,
    comment_counts: Rc<CommentCounts>,
    seen_comment_counts: Rc<CommentCounts>,
    surveys: Vec<Survey>,
    survey_answers: Vec<SurveyAnswers>,
    tabbar_bait_points: (bool, bool, bool, bool),
    page: Page,

    event_closing: bool,
    event_popup_size: Option<usize>,
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
                Some("friends") => link2.send_message(Msg::SilentSetPage(Page::Friends)),
                Some("notifications") => link2.send_message(Msg::SilentSetPage(Page::Notifications)),
                Some("r") => link2.send_message(Msg::SilentSetPage(Page::Rick)),
                Some(event) if event.starts_with("event/") => {
                    let eid = event[6..].to_string();
                    link2.send_message(Msg::SilentSetPage(Page::Event { eid }))
                }
                Some(survey) if survey.starts_with("survey/") => link2.send_message(Msg::SilentSetPage(Page::Survey { sid: survey[7..].to_string() })),
                Some(agenda) if agenda.starts_with("friend-agenda/") => link2.send_message(Msg::SilentSetPage(Page::FriendAgenda { pseudo: agenda[14..].to_string() })),
                _ if e.state().is_null() => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                _ => alert(format!("Unknown pop state: {:?}", e.state())),
            }
        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();

        // Update data
        let events = CachedData::init(ctx.link().clone()).unwrap_or_default();
        let user_info: Option<UserInfo> = CachedData::init(ctx.link().clone());
        let friends = CachedData::init(ctx.link().clone());
        let friends_events = FriendsEvents::init();
        let comment_counts = CachedData::init(ctx.link().clone()).unwrap_or_default();
        let survey_response: SurveyResponse = CachedData::init(ctx.link().clone()).unwrap_or_default();
        let surveys = survey_response.surveys;
        let survey_answers = survey_response.my_answers;
        let announcements = match &user_info {
            Some(user_info) => {
                let mut announcements: Vec<AnnouncementDesc> = CachedData::init(ctx.link().clone()).unwrap_or_default();
                announcements.retain(|a| a.target.as_ref().map(|t| user_info.groups.matches(t)).unwrap_or(true));
                announcements
            }
            None => Vec::new(),
        };

        // Load seen comment counts
        let local_storage = window().local_storage().unwrap().unwrap();
        let mut seen_comment_counts = Rc::new(HashMap::new());
        'try_load: {
            let Ok(Some(data)) = local_storage.get("seen_comment_counts") else { break 'try_load };
            let Ok(data) = serde_json::from_str::<HashMap<String, usize>>(&data) else { break 'try_load };
            seen_comment_counts = Rc::new(data);
        }
    
        // Open corresponding page
        let path = window().location().pathname().unwrap_or_default();
        let page = match path.as_str().trim_end_matches('/') {
            "/settings" => Page::Settings,
            "/friends" => Page::Friends,
            "/notifications" => Page::Notifications,
            event if event.starts_with("/event/") => {
                let eid = event[7..].to_string();
                let link2 = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    sleep(Duration::from_millis(100)).await;
                    link2.send_message(Msg::SetPage(Page::Event { eid }));
                });
                Page::Agenda
            }
            survey if survey.starts_with("/survey/") => Page::Survey { sid: survey[8..].to_string() },
            friend_agenda if friend_agenda.starts_with("/friend-agenda/") => Page::FriendAgenda { pseudo: friend_agenda[15..].to_string() },
            "/agenda" => match window().location().hash() { // For compatibility with old links
                Ok(hash) if hash == "#settings" => Page::Settings,
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
            let now = now();
            if let Some(survey_to_open) = surveys.iter().find(|s| s.required && s.start_ts <= now && s.end_ts >= now) {
                if !survey_answers.iter().any(|a| a.id == survey_to_open.id) {
                    ctx.link().send_message(Msg::SetPage(Page::Survey { sid: survey_to_open.id.clone() }));
                }
            }
        }

        // Get notification tracker
        let mut notifications = LocalNotificationTracker::load();
        notifications.add_announcements(&announcements);
        notifications.add_surveys(&surveys);

        // Set TabBar bait points
        let tabbar_bait_points = (
            false,
            friends.as_ref().map(|f: &FriendLists| !f.incoming.is_empty()).unwrap_or(false),
            notifications.has_unread(),
            false
        );

        Self {
            events: Rc::new(events),
            user_info: Rc::new(user_info),
            announcements: Rc::new(announcements),
            notifications: Rc::new(RefCell::new(notifications)),
            friends: Rc::new(friends),
            friends_events,
            comment_counts: Rc::new(comment_counts),
            seen_comment_counts,
            surveys,
            survey_answers,
            tabbar_bait_points,
            page,
            event_closing: false,
            event_popup_size: None,
        }
    }

    /// Most of the messages handled in the function are sent by the data loader to update the data or report an error.
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::UpdateFriends(friends) => {
                // Detect changes to add bait point
                if let Some(old_friends) = self.friends.as_ref() {
                    for new_incoming in &friends.incoming {
                        if !old_friends.incoming.contains(new_incoming) {
                            self.tabbar_bait_points.1 = true;
                        }
                    }
                }

                friends.save();
                self.friends = Rc::new(Some(friends));
                
                matches!(self.page, Page::Friends | Page::Event { .. }) || self.tabbar_bait_points.1
            },
            AppMsg::FriendsEventsSuccess { uid, events } => {
                self.friends_events.insert(uid, events);
                matches!(self.page, Page::FriendAgenda { .. } )
            },
            AppMsg::AnnouncementsSuccess(mut announcements) => {
                // Filter announcements
                if let Some(user_info) = self.user_info.deref().as_ref() {
                    announcements.retain(|a| a.target.as_ref().map(|t| user_info.groups.matches(t)).unwrap_or(true));
                } else {
                    return false;
                }

                // Add to notifications
                let mut notifications = self.notifications.borrow_mut();
                notifications.add_announcements(&announcements);
                self.tabbar_bait_points.2 = notifications.has_unread();

                self.announcements = Rc::new(announcements);
                
                matches!(self.page, Page::Notifications) || self.tabbar_bait_points.2
            },
            AppMsg::SurveysSuccess(surveys, survey_answers) => {
                // Add to notifications
                let mut notifications = self.notifications.borrow_mut();
                notifications.add_surveys(&surveys);
                self.tabbar_bait_points.2 = notifications.has_unread();

                self.surveys = surveys;
                self.survey_answers = survey_answers;

                // Automatically open survey if one is available and required
                let now = now();
                if let Some(survey) = self.surveys.iter().find(|s| s.required && s.start_ts <= now && s.end_ts >= now && !self.survey_answers.iter().any(|a| a.id == s.id)) {
                    ctx.link().send_message(Msg::SetPage(Page::Survey { sid: survey.id.clone() }));
                }
                
                matches!(self.page, Page::Survey { .. }) || self.tabbar_bait_points.2
            },
            AppMsg::ScheduleSuccess(events) => {
                self.events = Rc::new(events);
                matches!(self.page, Page::Agenda | Page::Event { .. })
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

                // Update events if user groups changed
                if let Some(old_user_info) = self.user_info.as_ref() {
                    if old_user_info.groups != user_info.groups {
                        self.events = Rc::new(Vec::new());
                        <Vec<RawEvent>>::refresh(ctx.link().clone());
                        should_refresh = true;
                    }
                }

                // Set new user info
                user_info.save();
                self.user_info = Rc::new(Some(user_info));

                should_refresh
            },
            Msg::FriendsSuccess(friends) => {
                self.friends = Rc::new(Some(friends));
                matches!(self.page, Page::Friends)
            },
            Msg::CommentCountsSuccess(comment_counts) => {
                self.comment_counts = Rc::new(comment_counts);
                matches!(self.page, Page::Agenda)
            }
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
                // Remove bait points
                match page {
                    Page::Agenda => self.tabbar_bait_points.0 = false,
                    Page::Friends => self.tabbar_bait_points.1 = false,
                    Page::Notifications => self.tabbar_bait_points.2 = false,
                    Page::Settings => self.tabbar_bait_points.3 = false,
                    _ => (),
                }

                // Mark notifications as read upon leaving the notifications page
                if let Page::Notifications = self.page {
                    self.notifications.borrow_mut().mark_all_as_read();
                }

                // FIXME TODO
                // Prevent user to go on an event page from the friend-agenda page as it is not supported
                if matches!(self.page, Page::FriendAgenda { .. }) && matches!(page, Page::Event { .. }) {
                    return false;
                }

                let document = window().doc();
                if let Page::Event { eid } = &page {
                    let should_mark_as_seen = self.comment_counts.get(eid).copied().unwrap_or_default() != self.seen_comment_counts.get(eid).copied().unwrap_or(0);
                    let eid2 = eid.clone();
                    if !matches!(self.page, Page::Event { .. }) || self.event_popup_size.is_none() {
                        if let Some(day_el) = document.get_element_by_id("day0") {
                            let rect = day_el.get_bounding_client_rect();
                            self.event_popup_size = Some((width() as f64 - rect.width() - 2.0 * rect.left()) as usize)
                        }
                    }
                    let app_link = ctx.link().clone();
                    spawn_local(async move {
                        window().doc().body().unwrap().set_attribute("style", "overflow: hidden").unwrap();
                        sleep(Duration::from_millis(500)).await;
                        window().doc().body().unwrap().remove_attribute("style").unwrap();
                        if should_mark_as_seen {
                            app_link.send_message(Msg::MarkCommentsAsSeen(eid2));
                        }
                    });
                }
                if matches!((&self.page, &page), (Page::Event { .. }, Page::Agenda)) && !self.event_closing {
                    self.event_closing = true;
                    let link = ctx.link().clone();
                    spawn_local(async move {
                        window().doc().body().unwrap().set_attribute("style", "overflow: hidden").unwrap();
                        sleep(Duration::from_millis(500)).await;
                        link.send_message(Msg::SetPage(Page::Agenda));
                        window().doc().body().unwrap().remove_attribute("style").unwrap();
                    });
                    return true;
                }
                if matches!(&page, Page::Agenda) {
                    self.event_closing = false;
                }
                let (data, title) = page.data_and_title();
                if let Ok(history) = window().history() {
                    let _ = history.push_state_with_url(&JsValue::from_str(&data), title, Some(&format!("/{data}")));
                }
                document.set_title(title);
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
            AppMsg::MarkCommentsAsSeen(eid) => {
                let val = self.comment_counts.get(&eid).copied().unwrap_or_default();
                let mut seen_comment_counts = self.seen_comment_counts.deref().clone();
                seen_comment_counts.retain(|eid,_| self.events.iter().any(|e| e.eid == *eid));
                seen_comment_counts.insert(eid, val);
                self.seen_comment_counts = Rc::new(seen_comment_counts);
                let local_storage = window().local_storage().unwrap().unwrap();
                let _ = local_storage.set("seen_comment_counts", &serde_json::to_string(&self.seen_comment_counts.deref()).unwrap());
                true
            }
        }
    }
    
    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        crate::colors::COLORS_CHANGED.store(false, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.page {
            Page::Agenda => html!(<>
                <Agenda
                    events={Rc::clone(&self.events)}
                    app_link={ctx.link().clone()}
                    user_info={Rc::clone(&self.user_info)}
                    friends={Rc::clone(&self.friends)}
                    comment_counts={Rc::clone(&self.comment_counts)}
                    seen_comment_counts={Rc::clone(&self.seen_comment_counts)} />
                <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
            </>),
            Page::Event { eid } => {
                let event = match self.events.iter().find(|e| e.eid == *eid) {
                    Some(event) => event.to_owned(),
                    None => {
                        ctx.link().send_message(Msg::SetPage(Page::Agenda));
                        return html!();
                    }
                };
                html!(<>
                    <Agenda
                        events={Rc::clone(&self.events)}
                        app_link={ctx.link().clone()}
                        popup={Some((event, self.event_closing, self.event_popup_size.to_owned()))}
                        friends={Rc::clone(&self.friends)}
                        user_info={Rc::clone(&self.user_info)}
                        comment_counts={Rc::clone(&self.comment_counts)}
                        seen_comment_counts={Rc::clone(&self.seen_comment_counts)} />
                    <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
                </>)
            },
            Page::Friends => html!(<>
                <FriendsPage friends={Rc::clone(&self.friends)} app_link={ctx.link().clone()} />
                <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
            </>),
            Page::FriendAgenda { pseudo } => {
                let email = format!("{pseudo}@insa-rouen.fr");
                let uid = match self.friends.deref().as_ref().and_then(|f| f.friends.iter().find(|f| f.0.email == *email)) {
                    Some(f) => f.0.uid,
                    None => return html!("404 friend not found"), // TODO 404 page
                };
                let events = self.friends_events.get_events(uid, ctx.link().clone()).unwrap_or_default();
                let profile_src = format!("https://api.dicebear.com/5.x/identicon/svg?seed={}", uid);
                html!(<>
                    <Agenda
                        events={events}
                        app_link={ctx.link().clone()}
                        profile_src={profile_src}
                        friends={Rc::clone(&self.friends)}
                        user_info={Rc::clone(&self.user_info)}
                        comment_counts={Rc::clone(&self.comment_counts)}
                        seen_comment_counts={Rc::clone(&self.seen_comment_counts)} />
                    <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
                </>)
            },
            Page::Notifications => html!(<>
                <NotificationsPage notifications={Rc::clone(&self.notifications)} />
                <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
            </>),
            Page::Settings => html!(<>
                <SettingsPage app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} />
                <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
            </>),
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
            Page::Rick => {
                let random = js_sys::Math::random();
                let rick = if random > 0.1 {"rick1"} else {"rick2"};
                let raw_html = format!(r#"<video class="rick" autoplay src="/assets/{rick}.mp4" style="width: 100%;">Never gonna give you up</video>"#);
                VNode::from_html_unchecked(raw_html.into())
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
