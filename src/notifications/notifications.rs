pub use crate::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct LocalNotificationTracker {
    notifications: HashMap<String, (bool, Notification)>,
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
                    button_target: Some(format!("/survey/{}", survey.id)),
                }));
            }
        }
        self.save()
    }

    pub fn unseen(&self) -> impl Iterator<Item = &Notification> {
        self.notifications.values().filter(|(seen, _)| !seen).map(|(id, notification)| notification)
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
    button_target: Option<String>,
}

pub struct NotificationsPage;

pub enum NotificationsMsg {
}

impl Component for NotificationsPage {
    type Message = NotificationsMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let unseen_texts = vec!["I will write some great placeholder calling for a total and complete shutdown of ", "Sondage hivernal"];
        let unseen_text_iter = unseen_texts.iter();
        let unseen_src_iter = unseen_texts.iter().map(|text| format!("https://api.dicebear.com/5.x/micah/svg?seed={text}", text = text.replace(" ", "+")));
        let unseen_alt_iter = unseen_texts.iter().map(|text| format!("Avatar of {text}"));
        let unseen_button_iter = unseen_texts.iter().map(|_| html!(<button class="friends-agenda-button">{"Participer"}</button>));

        let texts = vec!["I will write some great placeholder calling for a total and complete shutdown of ", "et cela"];
        let text_iter = texts.iter();
        let src_iter = texts.iter().map(|text| format!("https://api.dicebear.com/5.x/micah/svg?seed={text}", text = text.replace(" ", "+")));
        let alt_iter = texts.iter().map(|text| format!("Avatar of {text}"));
        let button_iter = texts.iter().map(|text| if 1 == 2 {Some(format!("Voir {text}", text = text))} else {None});

        template_html!("src/notifications/notifications.html", ...)
    }
}
