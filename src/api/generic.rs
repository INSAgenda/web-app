use serde::de::DeserializeOwned;
use crate::prelude::*;
use super::*;

pub trait CachedData: DeserializeOwned + Serialize {
    fn storage_key() -> &'static str;
    fn endpoint() -> &'static str;
    fn cache_duration() -> u64;
    fn force_reload(&self) -> bool { false }
    fn on_cache(&mut self) {}
    fn on_load(result: Result<Self, ApiError>, app_link: Scope<App>);

    fn save(&self) {
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;
        let storage_key = Self::storage_key();
        let local_storage = window().local_storage().unwrap().unwrap();
        let _ = local_storage.set(&format!("last_updated_{storage_key}"), &now.to_string());
        let _ = local_storage.set(&format!("cached_{storage_key}"), &serde_json::to_string(self).unwrap());    
    }

    fn init(app_link: Scope<App>) -> Option<Self> {
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;

        // Get cached
        let mut default = None;
        if let Some((last_updated, mut cached)) = load_cached::<Self>() {
            cached.on_cache();
            if last_updated > now - Self::cache_duration() as i64 && !cached.force_reload() {
                return Some(cached);
            }
            default = Some(cached);
        }
    
        // Update from server
        Self::refresh(app_link);
    
        default
    }

    fn refresh(app_link: Scope<App>) {
        wasm_bindgen_futures::spawn_local(async move {
            let result = load::<Self>().await;
            Self::on_load(result, app_link);
        });
    }
}

fn load_cached<T: CachedData>() -> Option<(i64, T)> {
    let local_storage = window().local_storage().unwrap().unwrap();
    let storage_key = T::storage_key();

    let Ok(Some(Ok(last_updated))) = local_storage.get(&format!("last_updated_{storage_key}")).map(|v| v.map(|v| v.parse())) else { return None };
    if storage_key == "events" { // Temporary
        if last_updated < 1684973740 {
            return None;
        }
    }
    let Ok(Some(cached_str)) = local_storage.get(&format!("cached_{storage_key}")) else { return None };
    let Ok(cached) = serde_json::from_str::<T>(&cached_str) else { return None };

    Some((last_updated, cached))
}

async fn load<T: CachedData>() -> Result<T, ApiError> {
    let (api_key, counter) = get_login_info();

    let request = Request::new_with_str(T::endpoint())?;
    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;

    let resp = JsFuture::from(window().fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;

    if resp.status() == 400 || resp.status() == 500 {
        let text = JsFuture::from(resp.text()?).await?;
        let error: KnownApiError = serde_json::from_str(&text.as_string().unwrap()).map_err(|e| ApiError::Unknown(JsValue::from_str(&e.to_string())))?;
        return Err(ApiError::Known(error));
    }

    let text = JsFuture::from(resp.text()?).await?;
    let value: T = serde_json::from_str(&text.as_string().unwrap()).map_err(|e| ApiError::Unknown(JsValue::from_str(&e.to_string())))?;
    value.save();

    Ok(value)
}

impl CachedData for Vec<RawEvent> {
    fn storage_key() ->  &'static str { "events" }
    fn endpoint() ->  &'static str { "/api/schedule" }
    fn cache_duration() -> u64 { 3600 / 2 }
    fn force_reload(&self) -> bool { self.is_empty() }
    fn on_cache(&mut self) { self.sort_by_key(|e| e.start_unixtime); }
    fn on_load(result: Result<Self, ApiError>, app_link: Scope<App>) {
        match result {
            Ok(mut events) => {
                events.sort_by_key(|e| e.start_unixtime);
                app_link.send_message(AppMsg::ScheduleSuccess(events));
            },
            Err(e) => app_link.send_message(AppMsg::ScheduleFailure(e)),
        }
    }
}

impl CachedData for UserInfo {
    fn storage_key() ->  &'static str { "user_info" }
    fn endpoint() ->  &'static str { "/api/user-info" }
    fn cache_duration() -> u64 { 3600*6 }
    fn on_load(result: Result<Self, ApiError>, app_link: Scope<App>) {
        match result {
            Ok(user_info) => {
                set_sentry_user_info(&user_info.email.0);
                app_link.send_message(AppMsg::UserInfoSuccess(user_info))
            },
            Err(e) => app_link.send_message(AppMsg::ApiFailure(e)),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SurveyResponse {
    pub surveys: Vec<Survey>,
    pub my_answers: Vec<SurveyAnswers>,
}

impl CachedData for SurveyResponse {
    fn storage_key() ->  &'static str { "surveys" }
    fn endpoint() ->  &'static str { "/api/surveys" }
    fn cache_duration() -> u64 { 3600*6 }
    fn on_cache(&mut self) { self.surveys.sort_by_key(|e| e.start_ts); }
    fn on_load(result: Result<Self, ApiError>, app_link: Scope<App>) {
        match result {
            Ok(mut val) => {
                val.surveys.sort_by_key(|e| e.start_ts);
                app_link.send_message(AppMsg::SurveysSuccess(val.surveys, val.my_answers));
            },
            Err(e) => app_link.send_message(AppMsg::ApiFailure(e)),
        }
    }
}

impl CachedData for FriendLists {
    fn storage_key() ->  &'static str { "friends" }
    fn endpoint() ->  &'static str { "/api/friends/" }
    fn cache_duration() -> u64 { 10 }
    fn on_load(result: Result<Self, ApiError>, app_link: Scope<App>) {
        match result {
            Ok(val) => app_link.send_message(AppMsg::FriendsSuccess(val)),
            Err(e) => app_link.send_message(AppMsg::ApiFailure(e)),
        }
    }
}

pub type CommentCounts = HashMap<String, usize>;

impl CachedData for CommentCounts {
    fn storage_key() ->  &'static str { "comment_counts" }
    fn endpoint() ->  &'static str { "/api/textbook-course-ids" }
    fn cache_duration() -> u64 { 3600 }
    fn on_load(result: Result<Self, ApiError>, app_link: Scope<App>) {
        match result {
            Ok(val) => app_link.send_message(AppMsg::CommentCountsSuccess(val)),
            Err(e) => app_link.send_message(AppMsg::ApiFailure(e)),
        }
    }
}

#[derive(Serialize, Deserialize)]

pub struct WifiSettings {
    pub ssid: String,
    pub password: String,
}

impl CachedData for WifiSettings {
    fn storage_key() ->  &'static str { "wifi_settings" }
    fn endpoint() ->  &'static str { "/api/get-wifi" }
    fn cache_duration() -> u64 { 3600 }
    fn on_load(result: Result<Self, ApiError>, app_link: Scope<App>) {
        match result {
            Ok(val) => app_link.send_message(AppMsg::WiFiSuccess(val)),
            Err(ApiError::Known(e)) if e.kind == "wifi_credentials_not_set" => (),
            Err(e) => app_link.send_message(AppMsg::ApiFailure(e)),
        }
    }
}
