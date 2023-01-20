mod logout;
pub(crate) use logout::*;
mod error;
pub use error::*;
mod gen_code;
pub use gen_code::gen_code;
mod colors;
pub use colors::*;
mod generic;
pub use generic::*;
mod friends;
pub use friends::*;

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
    req_init.body(Some(&JsValue::from_str(&body)));

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
            let json = JsFuture::from(response.json()?).await?;
            Err(ApiError::from(json))
        },
        _ => Err(ApiError::Unknown(response.into()))
    }
}

pub async fn api_get<T: DeserializeOwned>(endpoint: &str) -> Result<T, ApiError> {
    let (api_key, counter) = get_login_info();

    let mut req_init = web_sys::RequestInit::new();
    req_init.method("GET");

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
            Ok(serde_json::from_str(&text).unwrap())
        }
        400 | 500 => {
            let text = JsFuture::from(response.text()?).await?;
            let text: String = text.as_string().unwrap();
            Err(ApiError::Known(serde_json::from_str(&text).unwrap()))
        },
        _ => Err(ApiError::Unknown(response.into()))
    }
}

/// Send a POST request to the API and update the counter
#[deprecated(note = "Use api_post instead")]
pub(crate) async fn post_api_request(endpoint: &str, mut request_init: RequestInit, headers: Vec<(&str, &str)>) -> Result<JsValue, JsValue> {
    let (api_key, counter) = get_login_info();
    
    request_init.method("POST");

    let request = Request::new_with_str_and_init(&format!("/api/{}", endpoint), &request_init).unwrap();

    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;
    for (header, value) in headers {
        request.headers().set(header, value)?;
    }
    JsFuture::from(window().fetch_with_request(&request)).await
}