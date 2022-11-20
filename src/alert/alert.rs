use crate::prelude::*;

/// Display an alert message to the user
fn alert_with_report(message: impl AsRef<str>, report: bool) {
    if report {
        sentry_report(message.as_ref());
    }

    let doc = window().doc();
    let error_container = doc.get_element_by_id("errors").unwrap();
    let alert = doc.create_element("div").unwrap();
    let alert: HtmlElement = alert.dyn_into().unwrap();
    alert.set_inner_text(message.as_ref());
    alert.set_class_name("alert");
    
    error_container.append_child(&alert).unwrap();
    
    let on_click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        let src  =  event.target().unwrap().dyn_into::<HtmlElement>().unwrap();
        if src.class_name() == "alert" {
            src.remove();
        }
    }) as Box<dyn FnMut(_)>);

    error_container.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref()).unwrap();
    on_click.forget();
}

pub fn alert(message: impl AsRef<str>) {
    alert_with_report(message, true);
}

pub fn alert_no_reporting(message: impl AsRef<str>) {
    alert_with_report(message, false);
}
