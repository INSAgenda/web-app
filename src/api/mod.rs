mod auth;
pub(crate) use auth::*;
mod error;
pub use error::*;
mod gen_code;
pub use gen_code::gen_code;
mod generic;
pub use generic::*;
mod friends;
pub use friends::*;
mod textbook;
pub use textbook::*;

use crate::prelude::*;

fn get_login_info() -> (u64, u64) {
    let local_storage = window().local_storage().unwrap().unwrap();
    let (api_key, counter) = match (local_storage.get("api_key").unwrap(), local_storage.get("counter").unwrap()) {
        (Some(api_key), Some(counter)) => (api_key.parse().expect("Invalid login data"), counter.parse().expect("Invalid login data")),
        _ => {
            window().location().replace("/login").unwrap();
            std::process::exit(0);
        }
    };
    local_storage.set("counter", &format!("{}", counter + 1)).unwrap();

    (api_key, counter)
}

/// Increase the counter, by a lot. Use when getting `counter_too_low` errors.
pub fn counter_to_the_moon() {
    let local_storage = window().local_storage().unwrap().unwrap();
    let counter: u64 = local_storage.get("counter").unwrap().unwrap().parse().unwrap();
    local_storage.set("counter", &(counter + 111).to_string()).unwrap();
}

pub async fn api_post<T: Serialize>(data: T, endpoint: &str) -> Result<(), ApiError> {
    let body = serde_json::to_string(&data).unwrap();
    let (api_key, counter) = get_login_info();

    let mut req_init = web_sys::RequestInit::new();
    req_init.method("POST");
    req_init.body(Some(&JsValue::from_str(&body)));

    let request = Request::new_with_str_and_init(&format!("/api/{endpoint}"), &req_init).unwrap();
    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;
    request.headers().set("Content-Type", "application/json")?;

    let response = JsFuture::from(window().fetch_with_request(&request)).await?;
    let response: web_sys::Response = response.clone().dyn_into().unwrap();

    match response.status() {
        200 => Ok(()),
        400 | 500 => {
            let json = JsFuture::from(response.json()?).await?;
            Err(ApiError::from(json))
        },
        _ => Err(ApiError::Unknown(response.into()))
    }
}

pub async fn api_post_form(body: &str, endpoint: &str) -> Result<(), ApiError> {
    let (api_key, counter) = get_login_info();

    let mut req_init = web_sys::RequestInit::new();
    req_init.method("POST");
    req_init.body(Some(&JsValue::from_str(body)));

    let request = Request::new_with_str_and_init(&format!("/api/{endpoint}"), &req_init).unwrap();
    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;
    request.headers().set("Content-Type", "application/x-www-form-urlencoded")?;

    let response = JsFuture::from(window().fetch_with_request(&request)).await?;
    let response: web_sys::Response = response.clone().dyn_into().unwrap();

    match response.status() {
        200 => Ok(()),
        400 | 500 => {
            let text = JsFuture::from(response.text()?).await?;
            let text: String = text.as_string().unwrap();
            Err(ApiError::Known(serde_json::from_str(&text).unwrap()))
        },
        _ => Err(ApiError::Unknown(response.into()))
    }
}

pub async fn api_get<T: DeserializeOwned>(endpoint: impl std::fmt::Display) -> Result<T, ApiError> {
    api_custom_method(endpoint, "GET").await
}

pub async fn api_delete<T: DeserializeOwned>(endpoint: impl std::fmt::Display) -> Result<T, ApiError> {
    api_custom_method(endpoint, "DELETE").await
}

async fn api_custom_method<T: DeserializeOwned>(endpoint: impl std::fmt::Display, method: &'static str) -> Result<T, ApiError> {
    let (api_key, counter) = get_login_info();

    let mut req_init = web_sys::RequestInit::new();
    req_init.method(method);

    let request = Request::new_with_str_and_init(&format!("/api/{endpoint}"), &req_init).unwrap();
    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;
    request.headers().set("Content-Type", "application/json")?;

    let response = JsFuture::from(window().fetch_with_request(&request)).await?;
    let response: web_sys::Response = response.clone().dyn_into().unwrap();

    match response.status() {
        200 => {
            let text = JsFuture::from(response.text()?).await?;
            let text: String = text.as_string().unwrap();
            if std::any::type_name::<T>() == "()" && text.is_empty() {
                return Ok(serde_json::from_str("null").unwrap());
            }
            match serde_json::from_str(&text) {
                Ok(data) => Ok(data),
                Err(e) => Err(ApiError::Unknown(format!("Failed to parse JSON: {e}").into())),
            }
        }
        400 | 500 => {
            let text = JsFuture::from(response.text()?).await?;
            let text: String = text.as_string().unwrap();
            match serde_json::from_str(&text) {
                Ok(data) => Err(ApiError::Known(data)),
                Err(e) => Err(ApiError::Unknown(format!("Failed to parse JSON: {e}").into())),
            }
        },
        _ => Err(ApiError::Unknown(response.into()))
    }
}
