use crate::prelude::*;


#[cfg(feature = "debug")]
const PIXELWAR_IFRAME_URL: &str = "http://localhost:8000";

#[cfg(not(feature = "debug"))]
const PIXELWAR_IFRAME_URL: &str = "https://insaplace.insagenda.fr/";

pub fn init_pixelwar(page: &Page, app_link: AppLink) -> web_sys::Element {
    // Start loading the iframe so that it is ready when the user clicks on the tab
    let iframe = window().doc().create_element("iframe").unwrap();
    iframe.set_attribute("id", "insaplace-iframe").unwrap();
    iframe.set_attribute("src",  PIXELWAR_IFRAME_URL).unwrap();
    window().doc().body().unwrap().append_child(&iframe).unwrap();
    if !matches!(page, Page::PixelWar) {
        iframe.set_attribute("style", "display: none").unwrap();
        // Listen for message
        let on_message = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if e.origin() != PIXELWAR_IFRAME_URL {
                return;
            }

            let data = e.data();
            let data: js_sys::Object = match data.dyn_into::<js_sys::Object>() {
                Ok(data) => data,
                Err(_) => {
                    log!("Received message from insaplace with invalid data");
                    return;
                }
            };
            let ty = match Reflect::get(&data, &JsValue::from_str("ty")) {
                Ok(ty) => match ty.as_string() {
                    Some(ty) => ty,
                    None => {
                        log!("Received message from insaplace with invalid type");
                        return;
                    }
                }
                Err(_) => {
                    log!("Received message from insaplace without type");
                    return;
                }
            };
            let data = match Reflect::get(&data, &JsValue::from_str("data")) {
                Ok(data) => data,
                Err(_) => {
                    log!("Received message from insaplace without data");
                    return;
                }
            };

            match ty.as_str() {
                "cookies" => {
                    let data = match data.dyn_into::<Array>() {
                        Ok(data) => data.to_vec(),
                        Err(_) => {
                            log!("Received message from insaplace with invalid cookies data");
                            return;
                        }
                    };
                    let user_id = match data.first().and_then(|v| v.as_string()) {
                        Some(user_id) => user_id,
                        None => {
                            log!("Received message from insaplace with invalid user_id");
                            return;
                        }
                    };
                    let user_token = match data.get(1).and_then(|v| v.as_string()) {
                        Some(user_token) => user_token,
                        None => {
                            log!("Received message from insaplace with invalid user_token");
                            return;
                        }
                    };
                    let validation_token = match data.get(2).and_then(|v| v.as_string()) {
                        Some(validation_token) => validation_token,
                        None => {
                            log!("Received message from insaplace with invalid validation_token");
                            return;
                        }
                    };
                    spawn_local(async move {
                        let url = format!("set-insaplace-cookies?user_id={user_id}&user_token={user_token}&validation_token={validation_token}");
                        match api_get::<()>(url).await {
                            Ok(_) => log!("Successfully set insaplace cookies"),
                            Err(e) => {
                                log!("Failed to set insaplace cookies: {e}");
                            }
                        };
                    });
                }
                ty => {
                    log!("Received message from insaplace with unknown type {ty}");
                }
            }

        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref()).unwrap();
        on_message.forget();
    }
    iframe
}
