use crate::prelude::*;

/// Display an alert message to the user
pub fn alert(message: impl AsRef<str>) {
    let doc = window().doc();
    let error_container = match doc.get_element_by_id("errors") {
        Some(container) => container,
        None => {
            log!("No error container found, cannot display error: {}", message.as_ref());
            return;
        }
    };
    for child in error_container.children().into_iter() {
        let child: HtmlElement = child.dyn_into().unwrap();
        if child.inner_text() == message.as_ref() {
            return;
        }
    }
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
