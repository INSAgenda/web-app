pub use crate::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct LocalNotificationTracker {
    pub(self) notifications: HashMap<String, (bool, Notification)>,
}

impl LocalNotificationTracker {
    fn try_load() -> Option<Self> {
        let local_storage = window().local_storage().unwrap()?;
        let announcement_tracker = local_storage.get("notification_tracker").unwrap()?;
        serde_json::from_str(&announcement_tracker).ok()
    }

    pub fn load() -> Self {
        Self::try_load().unwrap_or_else(|| Self {
            notifications: HashMap::new(),
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
            if !self.notifications.contains_key(&id) {
                let mut text = HashMap::new();
                if announcement.ty == ContentType::Text {
                    if let Some(content_fr) = &announcement.content_fr {
                        text.insert(String::from("fr"), content_fr.clone());
                    }
                    if let Some(content_en) = &announcement.content_en {
                        text.insert(String::from("en"), content_en.clone());
                    }
                } else {
                    continue;
                }
                self.notifications.insert(id, (false, Notification {
                    text,
                    image_src: String::from("/agenda/images/info.svg"),
                    image_alt: String::from("Information"),
                    ts: announcement.start_ts,
                    button_target: None,
                }));
            }
        }
        self.save()
    }

    pub fn add_surveys(&mut self, surveys: &[Survey]) {
        for survey in surveys {
            let id = format!("survey:{}", survey.id);
            if !self.notifications.contains_key(&id) {
                let mut text = HashMap::new();
                text.insert(String::from("fr"), format!("Un nouveau sondage a été publié: {}", survey.title));
                text.insert(String::from("en"), format!("A new survey has been published: {}", survey.title));
                self.notifications.insert(id, (false, Notification {
                    text,
                    image_src: String::from("/agenda/images/survey.svg"),
                    image_alt: String::from("Survey"),
                    ts: survey.start_ts as u64,
                    button_target: Some((format!("/survey/{}", survey.id), String::from("Participer"))),
                }));
            }
        }
        self.save()
    }

    pub fn unseen(&self) -> impl Iterator<Item = &Notification> {
        self.notifications.values().filter(|(seen, _)| !seen).map(|(_, notification)| notification)
    }

    pub fn seen(&self) -> impl Iterator<Item = &Notification> {
        self.notifications.values().filter(|(seen, _)| *seen).map(|(_, notification)| notification)
    }

    pub fn mark_seen(&mut self, id: &str) {
        if let Some((seen, _)) = self.notifications.get_mut(id) {
            *seen = true;
        }
        self.save()
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
    pub notifications: Rc<LocalNotificationTracker>,
}

impl PartialEq for NotificationsProps {
    fn eq(&self, other: &Self) -> bool { self.notifications.notifications.len() == other.notifications.notifications.len() }
}

pub enum NotificationsMsg {
}

impl Component for NotificationsPage {
    type Message = NotificationsMsg;
    type Properties = NotificationsProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let unseen_text_iter = ctx.props().notifications.unseen().map(|n| n.text.get("fr").unwrap_or(&String::from("")).clone());
        let unseen_src_iter = ctx.props().notifications.unseen().map(|n| n.image_src.clone());
        let unseen_alt_iter = ctx.props().notifications.unseen().map(|n| n.image_alt.clone());
        let unseen_button_iter = ctx.props().notifications.unseen().map(|n| n.button_target.as_ref().map(|(uri,text)| html!(<a class="friends-agenda-button" href={uri.to_owned()}>{text}</a>)));

        let text_iter = ctx.props().notifications.seen().map(|n| n.text.get("fr").unwrap_or(&String::from("")).clone());
        let src_iter = ctx.props().notifications.seen().map(|n| n.image_src.clone());
        let alt_iter = ctx.props().notifications.seen().map(|n| n.image_alt.clone());
        let button_iter = ctx.props().notifications.seen().map(|n| n.button_target.as_ref().map(|(uri,text)| html!(<a class="friends-agenda-button" href={uri.to_owned()}>{text}</a>)));

        template_html!("src/notifications/notifications.html", ...)
    }
}
