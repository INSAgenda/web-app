use crate::prelude::*;

use super::get_login_info;

pub async fn get_colors() -> Result<HashMap<String, String>, ApiError> {
    let (api_key, counter) = get_login_info();

    let request = Request::new_with_str("/api/colors")?;

    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;

    let resp = JsFuture::from(window().fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let text = JsFuture::from(resp.text()?).await?;
    let text: String = text.as_string().unwrap();

    if resp.status() == 400 || resp.status() == 500 {
        match serde_json::from_str::<KnownApiError>(&text) {
            Ok(error) => return Err(error.into()),
            Err(_) => return Err(ApiError::Unknown(text.into())),
        }
    }

    let colors: HashMap<String, String> = match serde_json::from_str(&text) {
        Ok(colors) => colors,
        _ => return Err(ApiError::Unknown(text.into())),
    };

    Ok(colors)
}
