use js_sys::encode_uri_component;

use super::*;

pub async fn get_friends() -> Result<FriendLists, ApiError> {
    api_get("friends/").await
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
    
        let Ok(Some(cached_str)) = local_storage.get("cached_friends_events") else { return Self::default() };
        let Ok(cached) = serde_json::from_str::<HashMap<i64, (u64, Vec<RawEvent>)>>(&cached_str) else { return Self::default() };

        let mut event_map = HashMap::new();
        for (uid, (time, events)) in cached {
            event_map.insert(uid, (time, Rc::new(events)));
        }
    
        Self { events: event_map }
    }

    fn save(&self) {
        // Sort by time, then take the 2 most recent
        let mut records = self.events.iter().map(|(uid, (time, events))| (*uid, (*time, events.as_ref().to_owned()))).collect::<Vec<(i64, (u64, Vec<RawEvent>))>>();
        records.sort_by_key(|(_, (time, _))| u64::MAX-time);
        records.truncate(2);
        
        // Save to local storage
        let local_storage = window().local_storage().unwrap().unwrap();
        let events = records.into_iter().map(|(id, (time, events))| (id, (time, events))).collect::<HashMap<i64, (u64, Vec<RawEvent>)>>();
        local_storage.set("cached_friends_events", &serde_json::to_string(&events).unwrap()).unwrap();
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
        events.sort_by_key(|event| event.start_unixtime);
        self.events.insert(uid, (now() as u64, Rc::new(events)));
        self.save();
    }

    pub fn get_events(&self, uid: i64, app_link: AppLink) -> Option<Rc<Vec<RawEvent>>> {
        let res = self.events.get(&uid).map(|(last_updated, events)| (last_updated, Rc::clone(events)));
        match res {
            Some((last_updated, events)) => {
                if now() - *last_updated as i64 > 5*3600 {
                    Self::update_friend(uid, app_link);
                }
                Some(events)
            },
            None => {
                Self::update_friend(uid, app_link);
                None
            }
        }
    }
}
