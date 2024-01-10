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

    fn to_notification(&self, now: u64) -> Notification {
        match self {
            NotificationSource::Announcement(announcement) => {
                let mut content = HashMap::new();
                match announcement.ty {
                    ContentType::Text => {
                        if let Some(content_fr) = &announcement.content_fr {
                            content.insert(String::from("fr"), html! { <p>{content_fr}</p> });
                        }
                        if let Some(content_en) = &announcement.content_en {
                            content.insert(String::from("en"), html! { <p>{content_en}</p> });
                        }
                    }
                    ContentType::Html => {
                        if let Some(content_fr) = &announcement.content_fr {
                            let content_fr = Html::from_html_unchecked(content_fr.to_owned().into());
                            content.insert(String::from("fr"), content_fr);
                        }
                        if let Some(content_en) = &announcement.content_en {
                            let content_en = Html::from_html_unchecked(content_en.to_owned().into());
                            content.insert(String::from("en"), content_en);
                        }
                    }
                };
                Notification {
                    content,
                    image_src: String::from("/agenda/images/info.svg"),
                    image_alt: String::from("Information"),
                    ts: announcement.start_ts,
                    button_target: None,
                }
            },
            NotificationSource::Survey(survey) => {
                let mut content = HashMap::new();
                content.insert(String::from("fr"), html! { <p>{format!("Un nouveau sondage a été publié: {}", survey.title)}</p> });
                content.insert(String::from("en"), html! { <p>{format!("A new survey has been published: {}", survey.title)}</p> });
                let mut button_target = None;
                if survey.start_ts as u64 <= now && survey.end_ts as u64 >= now {
                    button_target = Some((format!("/survey/{}", survey.id), String::from("Participer")));
                }
                Notification {
                    content,
                    image_src: String::from("/agenda/images/survey.svg"),
                    image_alt: String::from("Survey"),
                    ts: survey.start_ts as u64,
                    button_target,
                }
            },
        }
    }
}

pub struct Notification {
    content: HashMap<String, Html>,
    image_src: String,
    image_alt: String,
    ts: u64,
    button_target: Option<(String, String)>,
}

pub struct NotificationsPage;

#[derive(Clone, Properties)]
pub struct NotificationsProps {
    pub notifications: Rc<RefCell<LocalNotificationTracker>>,
    pub wifi_ssid : Rc<Option<String>>,
    pub wifi_password : Rc<Option<String>>,
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
        let now = now() as u64;
        let empty_html = html! {};
        
        let unseen = ctx.props().notifications.borrow().unseen().map(|source| source.to_notification(now)).collect::<Vec<_>>();
        let unseen_text_iter = unseen.iter().map(|n| n.content.get(SETTINGS.locale()).unwrap_or_else(|| n.content.get("fr").unwrap_or(&empty_html)).clone());
        let unseen_src_iter = unseen.iter().map(|n| n.image_src.clone());
        let unseen_alt_iter = unseen.iter().map(|n| n.image_alt.clone());
        let unseen_button_iter = unseen.iter().map(|n| n.button_target.as_ref().map(|(uri,text)| html!(<a class="friends-agenda-button" href={uri.to_owned()}>{text}</a>)));

        let seen = ctx.props().notifications.borrow().seen().map(|source| source.to_notification(now)).collect::<Vec<_>>();
        let text_iter = seen.iter().map(|n| n.content.get(SETTINGS.locale()).unwrap_or_else(|| n.content.get("fr").unwrap_or(&empty_html)).clone());
        let src_iter = seen.iter().map(|n| n.image_src.clone());
        let alt_iter = seen.iter().map(|n| n.image_alt.clone());
        let button_iter = seen.iter().map(|n| n.button_target.as_ref().map(|(uri,text)| html!(<a class="friends-agenda-button" href={uri.to_owned()}>{text}</a>)));

        let notifications_empty = unseen.is_empty() && seen.is_empty();

        let opt_wifi_ssid = ctx.props().wifi_ssid.as_deref().map(|s| s.to_owned());
        let opt_wifi_password = ctx.props().wifi_password.as_deref().map(|s| s.to_owned());

        template_html!("src/notifications/notifications.html", ...)
    }
}
