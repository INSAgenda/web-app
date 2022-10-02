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
    let json = JsFuture::from(resp.json()?).await?;

    if resp.status() == 400 || resp.status() == 500 {
        let error: KnownApiError = match json.into_serde() {
            Ok(error) => error,
            _ => return Err(ApiError::Unknown(json)),
        };
        return Err(error.into());
    }

    let colors: HashMap<String, String> = match json.into_serde() {
        Ok(colors) => colors,
        _ => return Err(ApiError::Unknown(json)),
    };

    Ok(colors)
}


pub async fn publish_colors(colors: &Vec<(String, String)>) -> Result<(), ApiError> {
    let mut init = web_sys::RequestInit::new();
    let body = serde_json::to_string(&colors).unwrap();
    init.body(Some(&JsValue::from_str(&body)));

    match post_api_request("colors", init, vec![]).await {
        Ok(resp_value) => {
            let response: web_sys::Response = resp_value.clone().dyn_into().unwrap();
            match response.status() {
                200 => Ok(()),
                400 | 500 => {
                    let json = JsFuture::from(response.json().unwrap()).await.unwrap();
                    Err(ApiError::from(json))
                },
                _ => Err(ApiError::Unknown(resp_value))
            }
        },
        Err(e) => Err(ApiError::Unknown(e)),
    }
 }