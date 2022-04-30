use super::*;

pub(crate) async fn logout()-> Result<(), ApiError> {
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
    local_storage.delete("api_key").unwrap();
    local_storage.delete("counter").unwrap();
    local_storage.delete("last_updated").unwrap();
    local_storage.delete("cached_events").unwrap();

    let resp = JsFuture::from(window().fetch_with_request(&request)).await?;
    let resp: Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;
    
    if resp.status() != 200 {
        let error: KnownApiError = json.into_serde().expect("JSON parsing issue");
        return Err(error.into());
    }
    Ok(())
}
