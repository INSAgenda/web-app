use crate::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KnownApiError {
    pub kind: String, origin: String, pub message_en: String, pub message_fr: String
}

impl std::fmt::Display for KnownApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let KnownApiError { kind, origin, message_en, message_fr } = self;
        match SETTINGS.lang() {
            Lang::French => write!(f, "{message_fr} ({kind} in {origin})"),
            Lang::English => write!(f, "{message_en} ({kind} in {origin})"),
        }
    }
}

pub enum ApiError {
    Known(KnownApiError),
    Unknown(JsValue),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Known(e) => e.fmt(f),
            ApiError::Unknown(e) => write!(f, "{:?}", e),
        }
    }
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
