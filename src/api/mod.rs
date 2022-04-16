mod load_events;
pub use load_events::*;
mod error;
pub use error::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, Request};

pub fn gen_code(api_key: u64, counter: u64) -> u64 {
    let mut key = (api_key + 143 * counter) as u128;
    for _ in 0..11 {
        key = key * key + 453;
        if key <= 0xffff_ffff {
            key += 0x4242424242424242424242424242;
        }
        key &= 0x0000_0000_ffff_ffff_ffff_ffff_0000_0000;
        key >>= 32;
    }
    key as u64
}

pub fn save_counter(counter: u64) {
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();
    local_storage.set("counter", &counter.to_string()).unwrap();
}

/// Send a POST request to the API and update the counter
/// 
/// debug: http://127.0.0.1/api/{endpoint}
/// release: https://insagenda.fr/api/{endpoint}
/// 
pub(crate) async fn post_api_request(endpoint: &str, request_init: RequestInit, headers: Vec<(&str, &str)>) -> Result<JsValue, JsValue>{
    let mut request_init = request_init;
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();

    let api_key = local_storage.get("api_key").map(|v| v.map(|v| v.parse()));
    let counter = local_storage.get("counter").map(|v| v.map(|v| v.parse()));
    let (api_key, counter) = match (api_key, counter) {
        (Ok(Some(Ok(api_key))), Ok(Some(Ok(counter)))) => (api_key, counter),
        _ => {
            window.location().replace("/login").unwrap();
            std::process::exit(0);
        },
    };
    
    request_init.method("POST");

    #[cfg(debug_assertions)]
    let request = Request::new_with_str_and_init(&format!("http://127.0.0.1:8080/api/{}", endpoint), &request_init).unwrap();
    #[cfg(not(debug_assertions))]
    let request = Request::new_with_str_and_init(&format!("https://insagenda.fr/api/{}", endpoint), &request_init).unwrap();

    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;
    for (header, value) in headers {
        request.headers().set(header, value)?;
    }
    let resp = JsFuture::from(window.fetch_with_request(&request)).await;
    save_counter(counter + 1);
    resp 

}