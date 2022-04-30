#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

pub async fn sleep(duration: std::time::Duration) {
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |yes, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &yes,
                duration.as_millis() as i32,
            )
            .unwrap();
    }))
    .await
    .unwrap();
}

pub fn window() -> web_sys::Window {
    // We use unsafe in order to call unwrap_unchecked. This simplifies the code and reduces the program size.
    // We know the unwrap will never fail because in the context of a website, window is always defined.
    unsafe {
        web_sys::window().unwrap_unchecked()
    }
}
