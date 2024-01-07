use crate::prelude::*;

// Ids are inserted at the begginning so that we can trim the list
#[derive(Serialize, Deserialize, Default)]
struct MastodonSeenIds {
    unseen_ids: Vec<String>,
    seen_ids: Vec<String>,
}

impl MastodonSeenIds {
    fn load() -> Self {
        let storage = window().local_storage().unwrap().unwrap();
        let Some(data) = storage.get_item("mastodon_seen_ids").unwrap() else { return Self::default() };
        serde_json::from_str(&data).unwrap_or_default()
    }

    fn save(&self) {
        let storage = window().local_storage().unwrap().unwrap();
        storage.set_item("mastodon_seen_ids", &serde_json::to_string(self).unwrap()).unwrap();
    }

    fn insert_new_unseen_ids(&mut self, mut new_ids: Vec<String>) {
        new_ids.extend(self.unseen_ids.drain(..));
        new_ids.truncate(1_000);
        self.unseen_ids = new_ids;
    }

    fn mark_all_seen(&mut self) {
        self.seen_ids.extend(self.unseen_ids.drain(..));
        self.seen_ids.truncate(1_000);
    }

    fn has_unseen_ids(&self) -> bool {
        !self.unseen_ids.is_empty()
    }
}

pub fn init_mastodon(page: &Page, app_link: AppLink) -> web_sys::Element {
    // Start loading the iframe so that it is ready when the user clicks on the tab
    let iframe = window().doc().create_element("iframe").unwrap();
    iframe.set_attribute("id", "mastodon-iframe").unwrap();
    iframe.set_attribute("src", "https://insagenda.fr/cas/login?service=https%3A%2F%2Fmastodon.insa.lol%2Fauth%2Fauth%2Fcas%2Fcallback%3Furl%3Dhttps%253A%252F%252Fmastodon.insa.lol%252Fauth%252Fsign_in").unwrap();
    window().doc().body().unwrap().append_child(&iframe).unwrap();
    if !matches!(page, Page::Mastodon) {
        iframe.set_attribute("style", "display: none").unwrap();
    }

    // Listen for message
    let mut skip = false;
    let on_message = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
        if skip {
            return;
        }

        if e.origin() != "https://mastodon.insa.lol/" {
            log!("Received message from unknown origin {e:?}");
        }
        let data = e.data();
        let data: js_sys::Array = match data.dyn_into::<js_sys::Array>() {
            Ok(data) => data,
            Err(_) => {
                log!("Received message with invalid data");
                return;
            }
        };
        let mut ids = Vec::new();
        for element in data {
            if let Some(id) = element.as_string() {
                ids.push(id);
            }
        }
        let mut mastodon_seen_ids = MastodonSeenIds::load();
        mastodon_seen_ids.insert_new_unseen_ids(ids);
        mastodon_seen_ids.save();
        if mastodon_seen_ids.has_unseen_ids() {
            app_link.send_message(AppMsg::MastodonNotification);
            skip = true;
        }
    }) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref()).unwrap();
    on_message.forget();
    
    iframe
}

pub fn mastodon_mark_all_seen() {
    let mut mastodon_seen_ids = MastodonSeenIds::load();
    mastodon_seen_ids.mark_all_seen();
    mastodon_seen_ids.save();
}
