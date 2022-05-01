use super::*;

pub fn load_cached_user_info() -> Option<(i64, UserInfo)> {
    let local_storage = window().local_storage().unwrap().unwrap();

    let last_updated = match local_storage.get("last_updated_user_info").map(|v| v.map(|v| v.parse())) {
        Ok(Some(Ok(last_updated))) => last_updated,
        _ => return None,
    };
    
    let user_info_str = match local_storage.get("cached_user_info") {
        Ok(Some(user_info_str)) => user_info_str,
        _ => return None,
    };

    let user_info = match serde_json::from_str(&user_info_str) {
        Ok(user_info) => user_info,
        _ => return None,
    };

    Some((last_updated, user_info))
}

fn save_cache(last_updated: i64, user_info: &UserInfo) {
    let local_storage = window().local_storage().unwrap().unwrap();

    let _ = local_storage.set("last_updated_user_info", &last_updated.to_string());
    let _ = local_storage.set("cached_user_info", &serde_json::to_string(&user_info).unwrap());
}

pub async fn load_user_info() -> Result<UserInfo, ApiError> {
    let (api_key, counter) = get_login_info();

    #[cfg(debug_assertions)]
    let request = Request::new_with_str("http://127.0.0.1:8080/api/user-info")?;
    #[cfg(not(debug_assertions))]
    let request = Request::new_with_str("https://insagenda.fr/api/user-info")?;

    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;

    let resp = JsFuture::from(window().fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;

    if resp.status() == 400 {
        let error: KnownApiError = json.into_serde().expect("JSON parsing issue");
        return Err(error.into());
    }

    let user_info = json.into_serde().expect("JSON parsing issue");

    let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;
    save_cache(now, &user_info);

    Ok(user_info)
}
