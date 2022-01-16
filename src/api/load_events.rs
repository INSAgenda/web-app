use super::{gen_code, save_counter, error::*};
use agenda_parser::Event;
use std::{sync::atomic::{AtomicU64, Ordering::Relaxed}, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, Response};

pub async fn load_events(api_key: u64, counter: Rc<AtomicU64>) -> Result<Vec<Event>, ApiError> {
    let request = Request::new_with_str("http://127.0.0.1:8080/api/schedule?start_timestamp=0&end_timestamp=9999999999999")?;
    let counter = counter.fetch_add(1, Relaxed);
    save_counter(counter + 1);
    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;

    let window = web_sys::window().unwrap();
    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;

    if resp.status() == 400 {
        let error: KnownApiError = json.into_serde().expect("JSON parsing issue");
        return Err(error.into());
    }

    let branch_info: Vec<Event> = json.into_serde().expect("JSON parsing issue");

    Ok(branch_info)
}
