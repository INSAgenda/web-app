use super::*;
use crate::prelude::*;

fn load_cached_groups() -> Option<(i64, Vec<GroupDesc>)> {
    let local_storage = window().local_storage().unwrap().unwrap();

    let last_updated = match local_storage.get("last_updated_groups").map(|v| v.map(|v| v.parse())) {
        Ok(Some(Ok(last_updated))) => last_updated,
        _ => return None,
    };
    
    let cached_groups_str = match local_storage.get("cached_groups") {
        Ok(Some(cached_groups_str)) => cached_groups_str,
        _ => return None,
    };

    let cached_groups = match serde_json::from_str::<Vec<GroupDesc>>(&cached_groups_str) {
        Ok(cached_events) => cached_events,
        _ => return None,
    };

    Some((last_updated, cached_groups))
}

fn save_cache(last_updated: i64, events: &[GroupDesc]) {
    let local_storage = window().local_storage().unwrap().unwrap();

    let _ = local_storage.set("last_updated_groups", &last_updated.to_string());
    let _ = local_storage.set("cached_groups", &serde_json::to_string(&events).unwrap());
}

async fn load_groups() -> Result<Vec<GroupDesc>, ApiError> {
    let request = Request::new_with_str("/config/groups.json")?;
    let resp = JsFuture::from(window().fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;

    if resp.status() != 200 {
        return Err(ApiError::Unknown(json));
    }

    let groups: Vec<GroupDesc> = match json.into_serde() {
        Ok(groups) => groups,
        _ => return Err(ApiError::Unknown(json)),
    };

    let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;
    save_cache(now, &groups);

    Ok(groups)
}

pub fn init_groups(now: DateTime<chrono_tz::Tz>, app_link: Scope<App>) -> Vec<GroupDesc> {
    // Get cached groups
    let mut groups = Vec::new();
    if let Some((last_updated, cached)) = load_cached_groups() {
        groups = cached;
        if last_updated > now.timestamp() - 3600*12 && !groups.is_empty() {
            return groups;
        }
    }
    
    // Update groups from server
    wasm_bindgen_futures::spawn_local(async move {
        match load_groups().await {
            Ok(groups) => app_link.send_message(AppMsg::GroupsSuccess(groups)),
            Err(e) => sentry_report(&e),
        }
    });

    groups
}
