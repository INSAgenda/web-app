use crate::prelude::*;


#[cfg(feature = "debug")]
const PIXELWAR_IFRAME_URL: &str = "http://127.0.0.1:8000";

#[cfg(not(feature = "debug"))]
const PIXELWAR_IFRAME_URL: &str = "https://insaplace.insagenda.fr";

#[derive(Serialize, Deserialize)]
struct InsaplaceCookies {
    user_id: String,
    user_token: String,
    validation_token: String,
    member_id: String,
}

pub fn init_pixelwar(page: &Page, app_link: AppLink) -> web_sys::Element {
    // Start loading the iframe so that it is ready when the user clicks on the tab
    let iframe = window().doc().create_element("iframe").unwrap();
    iframe.set_attribute("id", "insaplace-iframe").unwrap();
    iframe.set_attribute("src",  PIXELWAR_IFRAME_URL).unwrap();
    window().doc().body().unwrap().append_child(&iframe).unwrap();
    if !matches!(page, Page::PixelWar) {
        iframe.set_attribute("style", "display: none").unwrap();
        // Listen for message
        let mut skip = false;
        let on_message = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if skip {
                return;
            }
        };

        match ty.as_str() {
            "cookies" => {
                send_insaplace_message("getSatus", &JsValue::null());

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

                let send_insaplace_message = send_insaplace_message.clone();
                spawn_local(async move {
                    let url = format!("set-insaplace-cookies?user_id={user_id}&user_token={user_token}&validation_token={validation_token}");
                    match api_get::<()>(url).await {
                        Ok(_) => log!("Successfully set insaplace cookies"),
                        Err(e) => {
                            log!("Failed to set insaplace cookies: {e}");
                        }
                    };
                    match api_get::<Vec<(UserDesc, InsaplaceCookies)>>("get-insaplace-cookies").await {
                        Ok(mut r) => {
                            log!("Successfully got insaplace friend cookies");
                            
                            r.insert(0, (UserDesc::new(0, String::from("Vous")), InsaplaceCookies {
                                user_id: user_id.clone(),
                                user_token: user_token.clone(),
                                validation_token: validation_token.clone(),
                            }));
                            let usernames = Array::new();
                            let cookies = Array::new();
                            for (user, user_cookies) in r {
                                usernames.push(&JsValue::from_str(&user.get_username()));
                                let array = Array::new();
                                array.push(&JsValue::from_str(&user_cookies.user_id));
                                array.push(&JsValue::from_str(&user_cookies.user_token));
                                array.push(&JsValue::from_str(&user_cookies.validation_token));
                                cookies.push(&JsValue::from(array));
                            }
                            let data = js_sys::Object::new();
                            Reflect::set(&data, &JsValue::from_str("usernames"), &JsValue::from(usernames)).unwrap();
                            Reflect::set(&data, &JsValue::from_str("cookies"), &JsValue::from(cookies)).unwrap();
                            send_insaplace_message("cookies", &JsValue::from(data));
                            log!("Successfully sent insaplace friend cookies")
                        }
                        Err(e) => log!("Failed to get insaplace cookies: {e}"),
                    };                
                });
            }
            "canPlace" => {
                if let Some(data) = data.as_bool()  {
                    app_link.send_message(AppMsg::SetPixelLockedState(data));
                } else {
                    log!("Received message from insaplace with invalid canPlace data");
                }
            }
            ty => {
                log!("Received message from insaplace with unknown type {ty}");
            }
        }
    }) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref()).unwrap();
    on_message.forget();
    
    iframe
}
