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
#[path = "mastodon/mastodon.rs"]
mod mastodon;

mod util;
mod slider;
mod api;
mod prelude;
mod translation;
mod colors;

use mastodon::{init_mastodon, mastodon_mark_all_seen};
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
    Mastodon,
    Event { eid: String },
    Rick,
}

impl Page {
    fn data_and_title(&self) -> (String, &'static str) {
        match self {
            Page::Settings => (String::from("settings"), "Settings"),
            Page::Agenda => (String::from("agenda"), "Agenda"),
            Page::Friends => (String::from("friends"), "Friends"),
            Page::FriendAgenda { pseudo } => (format!("friend-agenda/{pseudo}"), "Friend agenda"),
            Page::Mastodon => (String::from("mastodon"), "Mastodon"),
            Page::Event { eid } => (format!("event/{eid}"), "Event"),
            Page::Rick => (String::from("r"), "Rick"),
        }
    }
}

/// A message that can be sent to the `App` component.
pub enum Msg {
    /// Switch page
    SetPage { page: Page, silent: bool },
    FetchColors(HashMap<String, String>),
    UpdateFriends(FriendLists),
    MarkCommentsAsSeen(String),
    MastodonNotification,

    // Data updating messages sent by the loader in /src/api/generic.rs
    UserInfoSuccess(UserInfo),
    FriendsSuccess(FriendLists),
    FriendsEventsSuccess{ uid: i64, events: Vec<RawEvent> },
    CommentCountsSuccess(CommentCounts),
    ApiFailure(ApiError),
    ScheduleSuccess(Vec<RawEvent>),
    ScheduleFailure(ApiError),
    WiFiSuccess(WifiSettings),
}

/// Methods for backward compatibility
#[allow(non_snake_case)]
impl Msg {
    fn SetPage(page: Page) -> Self {
        Msg::SetPage { page, silent: false }
    }

    fn SilentSetPage(page: Page) -> Self {
        Msg::SetPage { page, silent: true }
    }
}

/// The main component of the app.
/// Stores data that is shared between pages, as well as the page that is currently displayed.
pub struct App {
    user_info: Rc<Option<UserInfo>>,
    events: Rc<Vec<RawEvent>>,
    friends: Rc<Option<FriendLists>>,
    friends_events: FriendsEvents,
    comment_counts: Rc<CommentCounts>,
    seen_comment_counts: Rc<CommentCounts>,
    tabbar_bait_points: (bool, bool, bool, bool),
    page: Page,
    wifi_ssid : Rc<Option<String>>,
    wifi_password : Rc<Option<String>>,
    event_closing: bool,
    event_popup_size: Option<usize>,
    iframe: web_sys::Element,
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
                Some("mastodon") => link2.send_message(Msg::SilentSetPage(Page::Mastodon)),
                Some("r") => link2.send_message(Msg::SilentSetPage(Page::Rick)),
                Some(event) if event.starts_with("event/") => {
                    let eid = event[6..].to_string();
                    link2.send_message(Msg::SilentSetPage(Page::Event { eid }))
                }
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
        let wifi_settings: Option<WifiSettings> = CachedData::init(ctx.link().clone());

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
            "/mastodon" => Page::Mastodon,
            event if event.starts_with("/event/") => {
                let eid = event[7..].to_string();
                let link2 = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    sleep(Duration::from_millis(100)).await;
                    link2.send_message(Msg::SetPage(Page::Event { eid }));
                });
                Page::Agenda
            }
            friend_agenda if friend_agenda.starts_with("/friend-agenda/") => Page::FriendAgenda { pseudo: friend_agenda[15..].to_string() },
            "/agenda" => match window().location().hash() { // For compatibility with old links
                Ok(hash) if hash == "#settings" => Page::Settings,
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

        // Set TabBar bait points
        let tabbar_bait_points = (
            false,
            friends.as_ref().map(|f: &FriendLists| !f.incoming.is_empty()).unwrap_or(false),
            false,
            false
        );

        let iframe = init_mastodon(&page, ctx.link().clone());

        Self {
            events: Rc::new(events),
            user_info: Rc::new(user_info),
            friends: Rc::new(friends),
            friends_events,
            comment_counts: Rc::new(comment_counts),
            wifi_ssid: Rc::new(wifi_settings.as_ref().map(|w: &WifiSettings| w.ssid.clone())),
            wifi_password: Rc::new(wifi_settings.map(|w| w.password)),
            seen_comment_counts,
            tabbar_bait_points,
            page,
            event_closing: false,
            event_popup_size: None,
            iframe,
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
            AppMsg::ScheduleSuccess(events) => {
                self.events = Rc::new(events);
                matches!(self.page, Page::Agenda | Page::Event { .. })
            },
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

                should_refresh || matches!(self.page, Page::Settings)
            },
            Msg::FriendsSuccess(friends) => {
                self.friends = Rc::new(Some(friends));
                matches!(self.page, Page::Friends)
            },
            Msg::CommentCountsSuccess(comment_counts) => {
                self.comment_counts = Rc::new(comment_counts);
                matches!(self.page, Page::Agenda)
            }
            Msg::WiFiSuccess(wifi_settings) => {
                self.wifi_ssid = Rc::new(Some(wifi_settings.ssid));
                self.wifi_password = Rc::new(Some(wifi_settings.password));
                //matches!(self.page, Page::Notifications)
                // TODO: find out where it's going to be
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
            Msg::SetPage { page, silent } => {
                // Remove bait points
                match page {
                    Page::Agenda => self.tabbar_bait_points.0 = false,
                    Page::Friends => self.tabbar_bait_points.1 = false,
                    Page::Mastodon => {
                        if self.tabbar_bait_points.2 {
                            mastodon_mark_all_seen();
                            self.tabbar_bait_points.2 = false;
                        }
                    },
                    Page::Settings => self.tabbar_bait_points.3 = false,
                    _ => (),
                }

                // Change the display of the Mastodon iframe when the user switches on or off the Mastodon page
                if matches!(self.page, Page::Mastodon) && !matches!(page, Page::Mastodon) { // off
                    self.iframe.set_attribute("style", "display: none").unwrap();
                } else if !matches!(self.page, Page::Mastodon) && matches!(page, Page::Mastodon)  { // on
                    self.iframe.remove_attribute("style").unwrap();
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
                if !silent {
                    if let Ok(history) = window().history() {
                        let _ = history.push_state_with_url(&JsValue::from_str(&data), title, Some(&format!("/{data}")));
                    }
                }
                document.set_title(title);
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
            AppMsg::MastodonNotification => {
                if matches!(self.page, Page::Mastodon) {
                    mastodon_mark_all_seen();
                    false
                } else {
                    self.tabbar_bait_points.2 = true;
                    true
                }
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
            Page::Mastodon => html!(<>
                <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
            </>),
            Page::Settings => html!(<>
                <SettingsPage app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} />
                <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
            </>),
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
