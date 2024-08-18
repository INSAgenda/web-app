use crate::mastodon::{init_mastodon, mastodon_mark_all_seen};
use yew::virtual_dom::{VChild, VNode};

use crate::{prelude::*, settings::SettingsPage};

/// A message that can be sent to the `App` component.
pub enum AppMsg {
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
}

/// Methods for backward compatibility
#[allow(non_snake_case)]
impl AppMsg {
    pub fn SetPage(page: Page) -> Self {
        AppMsg::SetPage { page, silent: false }
    }

    pub fn SilentSetPage(page: Page) -> Self {
        AppMsg::SetPage { page, silent: true }
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
    colors: Rc<Colors>,
    seen_comment_counts: Rc<CommentCounts>,
    tabbar_bait_points: (bool, bool, bool, bool),
    page: Page,
    iframe: web_sys::Element,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Handle popstate events (back browser button)
        let link2 = ctx.link().clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::PopStateEvent| {
            let state = e.state().as_string();
            let page = state.map(|s| Page::from_path(&s)).unwrap_or(Page::Agenda);
            link2.send_message(AppMsg::SilentSetPage(page));
        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();

        // Update data
        let events = CachedData::init(ctx.link().clone()).unwrap_or_default();
        let user_info: Option<UserInfo> = CachedData::init(ctx.link().clone());
        let friends = CachedData::init(ctx.link().clone());
        let friends_events = FriendsEvents::init();
        let comment_counts = CachedData::init(ctx.link().clone()).unwrap_or_default();
        let colors = CachedData::init(ctx.link().clone()).unwrap_or_default();

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
        let page = Page::from_path(&path);

        // Set TabBar bait points
        let tabbar_bait_points = (
            false,
            friends.as_ref().map(|f: &FriendLists| !f.incoming.is_empty()).unwrap_or(false),
            false,
            false,
        );

        let iframe = init_mastodon(&page, ctx.link().clone());
        Self {
            events: Rc::new(events),
            user_info: Rc::new(user_info),
            friends: Rc::new(friends),
            friends_events,
            comment_counts: Rc::new(comment_counts),
            colors: Rc::new(colors),
            seen_comment_counts,
            tabbar_bait_points,
            page,
            iframe
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
                if self.events.len() <= 25 {
                    alert_no_reporting("Votre agenda semble quasiment vide. Cochez bien tous vos groupes dans les paramètres.");
                }
                self.events = Rc::new(events);
                matches!(self.page, Page::Agenda | Page::Event { .. })
            },
            AppMsg::UserInfoSuccess(user_info) => {
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
            AppMsg::FriendsSuccess(friends) => {
                self.friends = Rc::new(Some(friends));
                matches!(self.page, Page::Friends)
            },
            AppMsg::CommentCountsSuccess(comment_counts) => {
                self.comment_counts = Rc::new(comment_counts);
                matches!(self.page, Page::Agenda)
            },
            AppMsg::ScheduleFailure(api_error) => {
                api_error.handle_api_error();
                if self.events.is_empty() {
                    alert("Failed to fetch schedule");
                }
                false
            },
            AppMsg::ApiFailure(api_error) => {
                api_error.handle_api_error();
                false
            },
            AppMsg::SetPage { page, silent } => {
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
            AppMsg::FetchColors(new_colors) => {
                self.colors = Rc::new(new_colors);
                matches!(self.page, Page::Agenda | Page::Event { .. })
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
            },
            AppMsg::MastodonNotification => {
                if matches!(self.page, Page::Mastodon) {
                    mastodon_mark_all_seen();
                    false
                } else {
                    self.tabbar_bait_points.2 = true;
                    true
                }
            },
        }
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
                    seen_comment_counts={Rc::clone(&self.seen_comment_counts)}
                    colors={Rc::clone(&self.colors)} />
                <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
            </>),
            Page::Event { eid }  => {
                let event = match self.events.iter().find(|e| e.eid == *eid) {
                    Some(event) => event.to_owned(),
                    None => {
                        ctx.link().send_message(AppMsg::SetPage(Page::Agenda));
                        return html!();
                    }
                };

                html!(<>
                    <Popup
                        event={event.clone()}
                        app_link={ctx.link().clone()}
                        friends={Rc::clone(&self.friends)}
                        user_info={Rc::clone(&self.user_info)}
                        colors={Rc::clone(&self.colors)} />
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
                        seen_comment_counts={Rc::clone(&self.seen_comment_counts)}
                        colors={Rc::clone(&self.colors)} />
                    <TabBar app_link={ctx.link()} page={self.page.clone()} bait_points={self.tabbar_bait_points} />
                </>)
            },
            Page::Mastodon  => html!(<>
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
            }
        }
    }
}
