pub use crate::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct LocalNotificationTracker {
    pub(self) notifications: Vec<(String, bool, NotificationSource)>,
}

impl LocalNotificationTracker {
    fn try_load() -> Option<Self> {
        let local_storage = window().local_storage().unwrap()?;
        let announcement_tracker = local_storage.get("notification_tracker").unwrap()?;
        serde_json::from_str(&announcement_tracker).ok()
    }

    pub fn load() -> Self {
        Self::try_load().unwrap_or_else(|| Self {
            notifications: Vec::new(),
        })
    }

    fn save(&self) {
        let local_storage = window().local_storage().unwrap().unwrap();
        let announcement_tracker = serde_json::to_string(&self).unwrap();
        local_storage.set("notification_tracker", &announcement_tracker).unwrap();
    }

    pub fn add_announcements(&mut self, announcements: &[AnnouncementDesc]) {
        for announcement in announcements {
            let id = format!("announcement:{}", announcement.id);
            if !self.notifications.iter().any(|(i,_,_)| i == &id) {
                self.notifications.push((id, false, NotificationSource::Announcement(announcement.clone())));
            }
        }
        self.notifications.sort_by_key(|(_,_,n)| u64::MAX - n.ts());
        self.save()
    }

    pub fn add_surveys(&mut self, surveys: &[Survey]) {
        for survey in surveys {
            let id = format!("survey:{}", survey.id);
            if !self.notifications.iter().any(|(i,_,_)| i == &id) {
                self.notifications.push((id, false, NotificationSource::Survey(survey.clone())));
            }
        }
        self.notifications.sort_by_key(|(_,_,n)| u64::MAX - n.ts());
        self.save()
    }

    pub fn unseen(&self) -> impl Iterator<Item = &NotificationSource> {
        self.notifications.iter().filter(|(_,seen, _)| !seen).map(|(_,_,source)| source)
    }

    pub fn seen(&self) -> impl Iterator<Item = &NotificationSource> {
        self.notifications.iter().filter(|(_,seen, _)| *seen).map(|(_,_,source)| source)
    }

    pub fn mark_all_as_read(&mut self) {
        self.notifications.iter_mut().for_each(|(_,seen, _)| *seen = true);
        self.save()
    }

    pub fn has_unread(&self) -> bool {
        self.notifications.iter().any(|(_,seen, _)| !seen)
    }
}

#[derive(Serialize, Deserialize)]
pub enum NotificationSource {
    Announcement(AnnouncementDesc),
    Survey(Survey),
}

impl NotificationSource {
    fn ts(&self) -> u64 {
        match self {
            NotificationSource::Announcement(announcement) => announcement.start_ts,
            NotificationSource::Survey(survey) => survey.start_ts as u64,
        }
    }

    fn into_notification(&self, now: u64) -> Notification {
        match self {
            NotificationSource::Announcement(announcement) => {
                let mut text = HashMap::new();
                if announcement.ty == ContentType::Text {
                    if let Some(content_fr) = &announcement.content_fr {
                        text.insert(String::from("fr"), content_fr.clone());
                    }
                    if let Some(content_en) = &announcement.content_en {
                        text.insert(String::from("en"), content_en.clone());
                    }
                }
                Notification {
                    text,
                    image_src: String::from("/agenda/images/info.svg"),
                    image_alt: String::from("Information"),
                    ts: announcement.start_ts,
                    button_target: None,
                }
            },
            NotificationSource::Survey(survey) => {
                let mut text = HashMap::new();
                text.insert(String::from("fr"), format!("Un nouveau sondage a été publié: {}", survey.title));
                text.insert(String::from("en"), format!("A new survey has been published: {}", survey.title));
                let mut button_target = None;
                if survey.start_ts as u64 <= now && survey.end_ts as u64 >= now {
                    button_target = Some((format!("/survey/{}", survey.id), String::from("Participer")));
                }
                Notification {
                    text,
                    image_src: String::from("/agenda/images/survey.svg"),
                    image_alt: String::from("Survey"),
                    ts: survey.start_ts as u64,
                    button_target,
                }
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Notification {
    text: HashMap<String, String>,
    image_src: String,
    image_alt: String,
    ts: u64,
    button_target: Option<(String, String)>,
}

pub struct NotificationsPage;

#[derive(Clone, Properties)]
pub struct NotificationsProps {
    pub notifications: Rc<RefCell<LocalNotificationTracker>>,
}

impl PartialEq for NotificationsProps {
    fn eq(&self, other: &Self) -> bool { self.notifications.borrow().notifications.len() == other.notifications.borrow().notifications.len() }
}

pub enum NotificationsMsg {
}

impl Component for NotificationsPage {
    type Message = NotificationsMsg;
    type Properties = NotificationsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
        
        let unseen = ctx.props().notifications.borrow().unseen().map(|source| source.into_notification(now)).collect::<Vec<_>>();
        let unseen_text_iter = unseen.iter().map(|n| n.text.get("fr").unwrap_or(&String::from("")).clone());
        let unseen_src_iter = unseen.iter().map(|n| n.image_src.clone());
        let unseen_alt_iter = unseen.iter().map(|n| n.image_alt.clone());
        let unseen_button_iter = unseen.iter().map(|n| n.button_target.as_ref().map(|(uri,text)| html!(<a class="friends-agenda-button" href={uri.to_owned()}>{text}</a>)));

        let seen = ctx.props().notifications.borrow().seen().map(|source| source.into_notification(now)).collect::<Vec<_>>();
        let text_iter = seen.iter().map(|n| n.text.get("fr").unwrap_or(&String::from("")).clone());
        let src_iter = seen.iter().map(|n| n.image_src.clone());
        let alt_iter = seen.iter().map(|n| n.image_alt.clone());
        let button_iter = seen.iter().map(|n| n.button_target.as_ref().map(|(uri,text)| html!(<a class="friends-agenda-button" href={uri.to_owned()}>{text}</a>)));

        let notifications_empty = unseen.is_empty() && seen.is_empty();

        template_html!("src/notifications/notifications.html", ...)
    }
}
