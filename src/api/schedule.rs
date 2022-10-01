use super::*;
use crate::prelude::*;

pub fn load_cached_events() -> Option<(i64, Vec<RawEvent>)> {
    let local_storage = window().local_storage().unwrap().unwrap();

    let last_updated = match local_storage.get("last_updated").map(|v| v.map(|v| v.parse())) {
        Ok(Some(Ok(last_updated))) => last_updated,
        _ => return None,
    };
    
    let cached_events_str = match local_storage.get("cached_events") {
        Ok(Some(cached_events_str)) => cached_events_str,
        _ => return None,
    };

    let mut cached_events = match serde_json::from_str::<Vec<RawEvent>>(&cached_events_str) {
        Ok(cached_events) => cached_events,
        _ => return None,
    };
    cached_events.sort_by_key(|e| e.start_unixtime);

    Some((last_updated, cached_events))
}

fn save_cache(last_updated: i64, events: &[RawEvent]) {
    let local_storage = window().local_storage().unwrap().unwrap();

    let _ = local_storage.set("last_updated", &last_updated.to_string());
    let _ = local_storage.set("cached_events", &serde_json::to_string(&events).unwrap());
}

pub async fn load_events() -> Result<Vec<RawEvent>, ApiError> {
    let (api_key, counter) = get_login_info();

    let request = Request::new_with_str("/api/schedule")?;

    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;

    let resp = JsFuture::from(window().fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;

    if resp.status() == 400 {
        let error: KnownApiError = match json.into_serde() {
            Ok(error) => error,
            _ => return Err(ApiError::Unknown(json)),
        };
        return Err(error.into());
    }

    let mut events: Vec<RawEvent> = match json.into_serde() {
        Ok(events) => events,
        _ => return Err(ApiError::Unknown(json)),
    };
    events.sort_by_key(|e| e.start_unixtime);

    let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;
    save_cache(now, &events);

    Ok(events)
}
