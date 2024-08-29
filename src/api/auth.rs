use super::*;

pub(crate) async fn logout()-> Result<(), ApiError> {
    let window = window();
    let local_storage = window.local_storage().unwrap().unwrap();
    
    let mut init = web_sys::RequestInit::new();
    init.method("POST");
    let request = Request::new_with_str_and_init("/api/auth/logout", &init).unwrap();

    #[cfg(debug_assertions)]
    request.headers().set("X-Insa-Auth-Email", USER)?;
    
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
    
    if resp.status() != 200 {
        let text = JsFuture::from(resp.text()?).await?;
        let text: String = text.as_string().unwrap();
        match serde_json::from_str::<KnownApiError>(&text) {
            Ok(error) => Err(error.into()),
            Err(_) => Err(ApiError::Unknown(text.into())),
        }
    } else {
        Ok(())
    }
}
