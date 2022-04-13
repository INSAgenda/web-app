mod load_events;
pub use load_events::*;
mod error;
pub use error::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, Response};

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

fn save_counter(counter: u64) {
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();
    local_storage.set("counter", &counter.to_string()).unwrap();
}

pub(crate) async fn logout()-> Result<(), ApiError>{
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

    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;
    
    if resp.status() == 400 {
        let error: KnownApiError = json.into_serde().expect("JSON parsing issue");
        return Err(error.into());
    }
    Ok(())
}