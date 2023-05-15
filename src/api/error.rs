use std::iter::FromIterator;
use crate::{prelude::*, redirect};
use js_sys::{Reflect, Function, Array, Object};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KnownApiError {
    pub kind: String, 
    messages: Option<HashMap<String, String>>,
    message_en: Option<String>,
    message_fr: Option<String>,
    origin: Option<String>,
}

impl std::fmt::Display for KnownApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let KnownApiError { kind, messages, message_en, message_fr, .. } = self;
        if let Some(messages) = messages {
            let msg = messages.get(SETTINGS.locale()).unwrap_or(kind);
            write!(f, "{msg} ({kind})")    
        } else if let (Some(msg_fr), Some(msg_en)) = (message_fr.as_ref(), message_en.as_ref()) {
            let msg = if SETTINGS.locale() == "fr" { msg_fr } else { msg_en };
            write!(f, "{msg} ({kind})")    
        } else {
            write!(f, "{kind}")
        }
    }
}

#[derive(Debug)]
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
        if let (Ok(Some(kind)), Ok(message_en), Ok(message_fr), Ok(origin)) = (Reflect::get(&value, &"kind".into()).map(|v| v.as_string()), Reflect::get(&value, &"message_en".into()).map(|v| v.as_string()), Reflect::get(&value, &"message_fr".into()).map(|v| v.as_string()), Reflect::get(&value, &"origin".into()).map(|v| v.as_string())) {
            return ApiError::Known(KnownApiError { kind, messages: None, message_en, message_fr, origin })
        }
        ApiError::Unknown(value)
    }
}

impl From<KnownApiError> for ApiError {
    fn from(value: KnownApiError) -> Self {
        ApiError::Known(value)
    }
}

pub trait SentryReportable {
    fn to_sentry_js_value(self) -> JsValue;
}

impl SentryReportable for JsValue {
    fn to_sentry_js_value(self) -> JsValue {
        self
    }
}

impl SentryReportable for String {
    fn to_sentry_js_value(self) -> JsValue {
        JsValue::from_str(&self)
    }
}

impl SentryReportable for &str {
    fn to_sentry_js_value(self) -> JsValue {
        JsValue::from_str(self)
    }
}

impl SentryReportable for &ApiError {
    fn to_sentry_js_value(self) -> JsValue {
        match self {
            ApiError::Known(e) => {
                let KnownApiError { kind, messages, message_en, .. } = e;
                let obj = Object::new();
                Reflect::set(&obj, &"kind".into(), &kind.into()).unwrap();
                let messages = if let Some(msg) = messages {
                    msg.get("en").unwrap_or(&kind)
                } else if let Some(msg) = message_en {
                    msg
                } else {
                    kind
                };
                Reflect::set(&obj, &"messages".into(), &messages.into()).unwrap();
                obj.into()
            }
            ApiError::Unknown(e) => e.to_owned(),
        }
    }
}

pub fn sentry_report(error: impl SentryReportable) {
    match Reflect::get(&window(), &JsValue::from_str("Sentry")) {
        Ok(sentry) => {
            let capture_exception = match Reflect::get(&sentry, &JsValue::from_str("captureException")){
                Ok(capture_exception) => capture_exception,
                Err(_) => {log!("Impossible to get the sentry JsValue."); return},
            };
            
            let capture_exception: Function = match capture_exception.dyn_into(){
                Ok(capture_exception) => capture_exception,
                Err(_) => {log!("Impossible to get the sentry function."); return},
            };
            
            match Reflect::apply(&capture_exception, &sentry, &Array::from_iter([error.to_sentry_js_value()])){
                Ok(_) => {},
                Err(_) => log!("Impossible to call the sentry function."),
            }
        }
        Err(_) => log!("Sentry not found")
    }
}

pub fn set_sentry_user_info(email: &str) {
    use Reflect::*;
    fn set_sentry_user_info_dirty(email: &str) -> Result<(), &'static str> {
        let sentry = get(&window(), &JsValue::from_str("Sentry")).map_err(|_| "could not get Sentry.")?;
        let set_user = get(&sentry, &JsValue::from_str("setUser")).map_err(|_| "could not get Sentry.setUser.")?;
        let set_user = set_user.dyn_into::<Function>().map_err(|_| "Sentry.setUser isn't a function.")?;
        let obj = Object::new();
        set(&obj, &"email".into(), &email.into()).map_err(|_| "could not set email.")?;
        let array = Array::from_iter([obj]);
        apply(&set_user, &sentry, &array).map_err(|_| "could not call Sentry.setUser.")?;
        Ok(())
    }
    if let Err(e) = set_sentry_user_info_dirty(email) {
        log!("Could not set sentry user info: {}", e);
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
            ApiError::Known(error) if error.kind == "counter_too_low" => {
                log!("firewall_ban");
                alert("Vous envoyez trop de requêtes! Veuillez réessayer plus tard.");
            }
            ApiError::Known(error) => {
                sentry_report(self);
                log!("{}", error.to_string());
                alert_no_reporting(error.to_string());
                if error.kind == "invalid_api_key" || error.kind == "authentification_required" || error.kind == "api_key_does_not_exist" || error.kind == "api_key_expired" {
                    redirect("/login");
                }
            }
            ApiError::Unknown(error) => {
                sentry_report(self);
                log!("Failed to call api: {:?}", error);
            }
        }
    }
}
