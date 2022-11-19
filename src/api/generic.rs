use serde::de::DeserializeOwned;
use async_trait::async_trait;
use crate::prelude::*;
use super::*;

#[async_trait]
pub trait LocalStorageItem: DeserializeOwned + Serialize + Default + PartialEq {
    fn storage_key() -> &'static str;
    fn endpoint() -> &'static str;
    fn cache_duration() -> u64;
    fn on_load(result: Result<Self, ApiError>, agenda_link: Scope<Agenda>);

    fn init(agenda_link: Scope<Agenda>) -> Self {
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;

        // Get cached
        let mut default = Self::default();
        if let Some((last_updated, cached)) = load_cached() {
            if last_updated > now - Self::cache_duration() as i64 && cached != default {
                return cached;
            }
            default = cached;
        }
    
        // Update from server
        wasm_bindgen_futures::spawn_local(async move {
            let result = load(now).await;
            Self::on_load(result, agenda_link);
        });
    
        default
    }
}

fn load_cached<T: LocalStorageItem>() -> Option<(i64, T)> {
    let local_storage = window().local_storage().unwrap().unwrap();
    let storage_key = T::storage_key();

    let Ok(Some(Ok(last_updated))) = local_storage.get(&format!("last_updated_{storage_key}")).map(|v| v.map(|v| v.parse())) else { return None };
    let Ok(Some(cached_str)) = local_storage.get(&format!("cached_{storage_key}")) else { return None };
    let Ok(cached) = serde_json::from_str::<T>(&cached_str) else { return None };

    Some((last_updated, cached))
}

async fn load<T: LocalStorageItem>(now: i64) -> Result<T, ApiError> {
    let (api_key, counter) = get_login_info();
    let storage_key = T::storage_key();

    let request = Request::new_with_str(T::endpoint())?;
    request.headers().set(
        "Api-Key",
        &format!("{}-{}-{}", api_key, counter, gen_code(api_key, counter)),
    )?;

    let resp = JsFuture::from(window().fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;

    if resp.status() == 400 || resp.status() == 500 {
        let error: KnownApiError = match serde_wasm_bindgen::from_value(json) {
            Ok(error) => error,
            _ => return Err(ApiError::Unknown(JsValue::from("Invalid JSON of error"))),
        };
        return Err(error.into());
    }

    let value: T = match serde_wasm_bindgen::from_value(json) {
        Ok(value) => value,
        _ => return Err(ApiError::Unknown(JsValue::from("Invalid JSON"))),
    };

    let local_storage = window().local_storage().unwrap().unwrap();
    let _ = local_storage.set(&format!("last_updated_{storage_key}"), &now.to_string());
    let _ = local_storage.set(&format!("cached_{storage_key}"), &serde_json::to_string(&value).unwrap());

    Ok(value)
}
