use js_sys::encode_uri_component;

use super::*;

pub async fn get_friends() -> Result<FriendsLists, ApiError> {
    api_get("friends/").await
}

pub async fn new_confirmation_email() -> Result<(), ApiError> {
    api_get("auth/new-confirmation-email").await
}

pub async fn request_friend(email: String) -> Result<(), ApiError> {
    api_post_form(&format!("email={}", encode_uri_component(&email)), "friends/request").await
}

pub async fn accept_friend(uid: i64) -> Result<(), ApiError> {
    api_post_form(&format!("uid={uid}"), "friends/accept").await
}

pub async fn decline_friend(uid: i64) -> Result<(), ApiError> {
    api_post_form(&format!("uid={uid}"), "friends/decline").await
}

pub async fn remove_friend(uid: i64) -> Result<(), ApiError> {
    api_post_form(&format!("uid={uid}"), "friends/remove").await
}

pub async fn get_friends_schedule(uid: i64) -> Result<Vec<RawEvent>, ApiError> {
    api_get(&format!("schedule?uid={uid}")).await
}


#[derive(Default, Clone)]
pub struct FriendsEvents {
    events: HashMap<i64, (u64, Rc<Vec<RawEvent>>)>,
}

impl FriendsEvents {
    pub fn init() -> Self {
        let local_storage = window().local_storage().unwrap().unwrap();
    
        let Ok(Some(cached_str)) = local_storage.get(&format!("cached_friends_events")) else { return Self::default() };
        //let Ok(cached) = serde_json::from_str::<Self>(&cached_str) else { return Self::default() };
    
        Self::default()
    }

    fn save(&self) {
        // TODO garbage collection
        //let local_storage = window().local_storage().unwrap().unwrap();
        //local_storage.set(&format!("cached_friends_events"), &serde_json::to_string(self).unwrap()).unwrap();    
    }

    pub fn update_friend(uid: i64, app_link: AppLink) {
        spawn_local(async move {
            match get_friends_schedule(uid).await {
                Ok(events) => app_link.send_message(AppMsg::FriendsEventsSuccess { uid, events }),
                Err(err) => app_link.send_message(AppMsg::ApiFailure(err)),
            }
        })
    }

    pub fn insert(&mut self, uid: i64, mut events: Vec<RawEvent>) {
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
        events.sort_by_key(|event| event.start_unixtime);
        self.events.insert(uid, (now, Rc::new(events)));
        self.save();
    }

    pub fn get_events(&self, uid: i64, app_link: AppLink) -> Option<Rc<Vec<RawEvent>>> {
        let events = self.events.get(&uid).map(|(_, events)| Rc::clone(&events));
        if events.is_none() {
            Self::update_friend(uid, app_link);
        }
        events
    }
}
