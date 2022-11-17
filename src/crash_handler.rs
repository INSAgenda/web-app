const CRASH_PAGE: &str = include_str!("../templates/crash.html");

use js_sys::{Reflect::get, Function};
use crate::prelude::*;

pub fn init() {
    std::panic::set_hook(Box::new(|info| {
        let window = window();
        let doc = window.doc().document_element().unwrap();

        let mut payload: Option<String> = info.payload().downcast_ref().map(|v: &String| v.to_owned());
        if payload.is_none() {
            if let Some(p2) = info.payload().downcast_ref::<&'static str>() {
                payload = Some(p2.to_string());
            }
        }

        let mut message = match (payload, info.location()) {
            (Some(payload), Some(location)) => format!("web-app panicked at '{}', {}", payload, location),
            (Some(payload), None) => format!("web-app panicked at '{}'", payload),
            (None, Some(location)) => format!("web-app panicked, {}", location),
            (None, None) => format!("web-app panicked, {:?}", info),
        };

        let encode_uri_component = get(&window, &"encodeURIComponent".into()).unwrap();
        let encode_uri_component: Function = encode_uri_component.dyn_into().unwrap();
        let encoded_message = encode_uri_component.call1(&window, &JsValue::from_str(&message)).unwrap();
        let encoded_message = encoded_message.as_string().unwrap();
        let html = CRASH_PAGE.replace("[ENCODED MESSAGE]", &encoded_message);

        message = message.replace('\"', "&quot;");
        message = message.replace('<', "&lt;");
        message = message.replace('>', "&gt;");

        let html = html.replace("[MESSAGE]", &message);
        doc.set_inner_html(&html);
    }));
}
