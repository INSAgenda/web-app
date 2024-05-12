use crate::prelude::*;


#[cfg(feature = "debug")]
const PIXELWAR_IFRAME_URL: &str = "http://127.0.0.1:8000";

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
        let mut skip = false;
        let on_message = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if skip {
                return;
            }

            if e.origin() != PIXELWAR_IFRAME_URL {
                log!("Received message from unknown origin {e:?}");
            }
        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref()).unwrap();
        on_message.forget();
    }

    iframe
}