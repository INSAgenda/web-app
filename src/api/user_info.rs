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

    let user_info: UserInfo = match serde_json::from_str(&user_info_str) {
        Ok(user_info) => user_info,
        _ => return None,
    };
    set_sentry_user_info(&user_info.email.0);
    Some((last_updated, user_info))
}

pub fn save_user_info_cache(user_info: &UserInfo) {
    let local_storage = window().local_storage().unwrap().unwrap();

    let _ = local_storage.set("last_updated_user_info", &now_ts().to_string());
    let _ = local_storage.set("cached_user_info", &serde_json::to_string(&user_info).unwrap());
}

pub async fn load_user_info() -> Result<UserInfo, ApiError> {
    let (api_key, counter) = get_login_info();

    let request = Request::new_with_str("/api/user-info")?;

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

    let user_info: UserInfo = match json.into_serde() {
        Ok(user_info) => user_info,
        _ => return Err(ApiError::Unknown(json)),
    };
    set_sentry_user_info(&user_info.email.0);

    Ok(user_info)
}


