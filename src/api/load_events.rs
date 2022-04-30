use super::*;
use crate::prelude::*;

pub fn load_cache() -> Option<(i64, Vec<Event>)> {
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();

    let last_updated = match local_storage.get("last_updated").map(|v| v.map(|v| v.parse())) {
        Ok(Some(Ok(last_updated))) => last_updated,
        _ => return None,
    };
    
    let cached_events_str = match local_storage.get("cached_events") {
        Ok(Some(cached_events_str)) => cached_events_str,
        _ => return None,
    };

    let cached_events = match serde_json::from_str::<Vec<Event>>(&cached_events_str) {
        Ok(cached_events) => cached_events,
        _ => return None,
    };

    Some((last_updated, cached_events))
}

fn save_cache(last_updated: i64, events: &[Event]) {
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();

    let _ = local_storage.set("last_updated", &last_updated.to_string());
    let _ = local_storage.set("cached_events", &serde_json::to_string(&events).unwrap());
}

pub async fn load_events() -> Result<Vec<Event>, ApiError> {
    let (api_key, counter) = get_login_info();

    #[cfg(debug_assertions)]
    let request = Request::new_with_str("http://127.0.0.1:8080/api/schedule?start_timestamp=0&end_timestamp=9999999999999")?;
    #[cfg(not(debug_assertions))]
    let request = Request::new_with_str("https://insagenda.fr/api/schedule?start_timestamp=0&end_timestamp=9999999999999")?;

    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;

    let window = window().unwrap();
    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;

    if resp.status() == 400 {
        let error: KnownApiError = json.into_serde().expect("JSON parsing issue");
        return Err(error.into());
    }

    let events: Vec<Event> = json.into_serde().expect("JSON parsing issue");

    let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;
    save_cache(now, &events);

    Ok(events)
}
