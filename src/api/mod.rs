mod load_events;
pub use load_events::*;
mod logout;
pub use logout::*;
mod error;
pub use error::*;

use crate::prelude::*;

fn gen_code(api_key: u64, counter: u64) -> u64 {
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

/// Send a POST request to the API and update the counter
/// 
/// debug: http://127.0.0.1/api/{endpoint}
/// release: https://insagenda.fr/api/{endpoint}
/// 
pub(crate) async fn post_api_request(endpoint: &str, mut request_init: RequestInit, headers: Vec<(&str, &str)>) -> Result<JsValue, JsValue> {
    let (api_key, counter) = get_login_info();
    
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
    let resp = JsFuture::from(window().fetch_with_request(&request)).await;

    resp
}