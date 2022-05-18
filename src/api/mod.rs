mod load_events;
pub use load_events::*;
mod logout;
pub use logout::*;
mod user_info;
pub use user_info::*;
mod error;
pub use error::*;
mod gen_code;
pub use gen_code::gen_code;

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