use std::iter::FromIterator;
use crate::{prelude::*, redirect};
use js_sys::{Reflect, Function, Array};
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

impl KnownApiError {
    fn to_string_en(&self) -> String {
        let KnownApiError { kind, origin, message_en, message_fr: _ } = self;
        format!("{message_en} ({kind} in {origin})")
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

impl ApiError {
    fn to_string_en(&self) -> String {
        match self {
            ApiError::Known(e) => e.to_string_en(),
            ApiError::Unknown(e) => format!("{:?}", e),
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

pub fn sentry_report(error: JsValue) {
    match Reflect::get(&window(), &JsValue::from_str("Sentry")) {
        Ok(sentry) => {
            let capture_exception = Reflect::get(&sentry, &JsValue::from_str("captureException")).expect("captureException in Sentry");
            let capture_exception: Function = capture_exception.dyn_into().expect("captureException to be a function");
            Reflect::apply(&capture_exception, &sentry, &Array::from_iter([error])).expect("Failed to call captureException");
        }
        Err(_) => log!("Sentry not found")
    }
}

impl ApiError{
    /// Handle API errors and redirect the user to the login page if necessary
    pub fn handle_api_error(&self) {
        sentry_report(JsValue::from_str(&self.to_string_en()));

        match self {
            ApiError::Known(error) if error.kind == "counter_too_low" => {
                log!("Counter too low");
                counter_to_the_moon();
            }
            ApiError::Known(error) => {
                log!("{}", error.to_string());
                alert_no_reporting(error.to_string());
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
