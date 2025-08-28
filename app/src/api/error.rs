use std::iter::FromIterator;
use crate::{prelude::*, redirect};
use js_sys::{Reflect, Function, Array, Object};
use serde::Deserialize;

#[allow(dead_code)]
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
            ApiError::Unknown(e) => {
                match e.as_string() {
                    Some(s) => write!(f, "{}", s),
                    None => write!(f, "{:?}", e),
                }
            },
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
                log!("{}", error.to_string());
                alert(error.to_string());
                if error.kind == "invalid_api_key" || error.kind == "authentification_required" || error.kind == "api_key_does_not_exist" || error.kind == "api_key_expired" {
                    redirect("/login");
                }
            }
            ApiError::Unknown(error) => {
                log!("Failed to call api: {:?}", error);
            }
        }
    }
}
