use super::*;

pub(crate) async fn logout()-> Result<(), ApiError> {
    let window = window();
    let local_storage = window.local_storage().unwrap().unwrap();
    let (api_key, counter) = get_login_info();
    
    let mut init = web_sys::RequestInit::new();
    init.method("POST");
    #[cfg(debug_assertions)]
    let request = Request::new_with_str_and_init("http://127.0.0.1:8080/api/auth/logout", &init).unwrap();
    #[cfg(not(debug_assertions))]
    let request = Request::new_with_str_and_init("https://insagenda.fr/api/auth/logout", &init).unwrap();

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
        let error: KnownApiError = json.into_serde().expect("JSON parsing issue");
        return Err(error.into());
    }
    Ok(())
}
