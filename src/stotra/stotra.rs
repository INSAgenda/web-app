use std::sync::atomic::AtomicIsize;
use serde_json::Value;
use crate::prelude::*;

#[cfg(debug_assertions)]
pub const STOTRA_URL: &str = "http://localhost:5173";
#[cfg(not(debug_assertions))]
pub const STOTRA_URL: &str = "https://auth.insa.lol/cas/login?service=https%3A%2F%2Fstotra.insa.lol";

#[cfg(debug_assertions)]
pub const STOTRA_GET_PORTFOLIO_URL: &str = "http://localhost:5173/api/user/portfolio";
#[cfg(not(debug_assertions))]
pub const STOTRA_GET_PORTFOLIO_URL: &str = "https://stotra.insa.lol/api/user/portfolio";

static STOTRA_RANK_CACHED: AtomicIsize = AtomicIsize::new(0);

pub async fn get_stotra_rank() -> Result<Option<usize>, ApiError> {
    let request = Request::new_with_str(STOTRA_GET_PORTFOLIO_URL).unwrap();

    #[cfg(debug_assertions)]
    request.headers().set("X-Username", "test")?;

    request.headers().set("Content-Type", "application/json")?;

    let response = JsFuture::from(window().fetch_with_request(&request)).await?;
    let response: web_sys::Response = response.clone().dyn_into().unwrap();

    match response.status() {
        200 => {
            let text = JsFuture::from(response.text()?).await?;
            let text: String = text.as_string().unwrap();
            let values: HashMap<String, Value> = serde_json::from_str(&text).map_err(|e| ApiError::Unknown(JsValue::from_str(&e.to_string())))?;
            let rank = values.get("rank").ok_or(ApiError::Unknown(JsValue::from_str("Failed to get rank")))?;
            let rank = rank.as_f64().ok_or(ApiError::Unknown(JsValue::from_str("Failed to convert rank to usize")))? as isize;
            log!("storing");
            STOTRA_RANK_CACHED.store(rank, Ordering::Relaxed);
            log!("stored");
            if rank <= 0 {
                Ok(None)
            } else {
                Ok(Some(rank as usize))
            }
        },
        400 | 500 => {
            let json = JsFuture::from(response.json()?).await?;
            Err(ApiError::from(json))
        },
        _ => Err(ApiError::Unknown(response.into()))
    }
}

#[function_component(StotraRank)]
pub fn stotra_rank_component() -> Html {
    let (default_rank, must_fetch) = match STOTRA_RANK_CACHED.load(Ordering::Relaxed) {
        negative if negative < 0 => (None, false),
        0 => (None, true),
        positive => (Some(positive as usize), false),
    };
    
    let rank = use_state(|| default_rank);

    if must_fetch {
        let rank2 = rank.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(r) = get_stotra_rank().await {
                rank2.set(r)
            }
        });
    }

    match rank.as_ref() {
        Some(rank) => {
            html! {
                <div id="stotra-rank">{"#"}{ *rank }</div>
            }
        },
        None => {
            return html!()
        },
    }
}
