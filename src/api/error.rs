use crate::{prelude::*, redirect};
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

impl ApiError{
    /// Handle API errors and redirect the user to the login page if necessary
    pub fn handle_api_error(&self) {
        match self {
            ApiError::Known(error) if error.kind == "counter_too_low" => {
                log!("Counter too low");
                counter_to_the_moon();
            }
            ApiError::Known(error) => {
                log!("{}", error.to_string());
                alert(error.to_string());
                if error.kind == "invalid_api_key" || error.kind == "authentification_required" || error.kind == "api_key_does_not_exist" || error.kind == "expired" {
                    redirect("/login");
                }
            }
            ApiError::Unknown(error) => {
                log!("Failed to call api: {:?}", error);
                redirect("/login");
            }
        }
    }
}
