use wasm_bindgen::JsValue;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KnownApiError {
    kind: String, origin: String, message_en: String, message_fr: String
}

pub enum ApiError {
    Known(KnownApiError),
    Unknown(JsValue),
}

impl From<JsValue> for ApiError {
    fn from(value: JsValue) -> Self {
        ApiError::Unknown(value)
    }
}

impl From<KnownApiError> for ApiError {
    fn from(value: KnownApiError) -> Self {
        ApiError::Known(value)
    }
}
