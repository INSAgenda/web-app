use super::*;

pub async fn new_confirmation_email() -> Result<(), ApiError> {
    api_get("auth/new-confirmation-email").await
}

pub(crate) async fn logout()-> Result<(), ApiError> {
    let window = window();
    let local_storage = window.local_storage().unwrap().unwrap();
    let (api_key, counter) = get_login_info();
    
    let mut init = web_sys::RequestInit::new();
    init.method("POST");
    let request = Request::new_with_str_and_init("/api/auth/logout", &init).unwrap();

    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;
    let theme = local_storage.get("setting-theme").unwrap();
    let auto = local_storage.get("auto-theme").unwrap();
    local_storage.clear().unwrap();
    if let Some(theme) = theme {
        local_storage.set("setting-theme", &theme).unwrap();
    }
    if let Some(auto) = auto {
        local_storage.set("auto-theme", &auto).unwrap();
    }

    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;
    
    if resp.status() != 200 {
        let error: KnownApiError = match serde_wasm_bindgen::from_value(json.clone()) {
            Ok(error) => error,
            _ => return Err(ApiError::Unknown(json)),
        };
        return Err(error.into());
    }
    Ok(())
}
