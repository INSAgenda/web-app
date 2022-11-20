use wasm_bindgen::JsCast;
use web_sys::*;
use crate::prelude::*;

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

pub fn now_ts() -> i64 {
    (js_sys::Date::new_0().get_time() / 1000.0) as i64
}

pub trait HackTraitDocOnWindow {
    fn doc(&self) -> Document;
}

impl HackTraitDocOnWindow for Window {
    fn doc(&self) -> Document {
        unsafe {
            self.document().unwrap_unchecked()
        }
    }
}

pub trait HackTraitSelectedValueOnHtmlSelectElement {
    fn selected_value(&self) -> String;
}

impl HackTraitSelectedValueOnHtmlSelectElement for HtmlSelectElement {
    fn selected_value(&self) -> String {
        unsafe {
            self.selected_options().item(0).unwrap().dyn_into::<HtmlOptionElement>().unwrap_unchecked().value()
        }
    }
}

pub trait HackTraitHtmlCollectionIter {
    fn into_iter(self) -> HtmlCollectionIter;
}

impl HackTraitHtmlCollectionIter for HtmlCollection {
    fn into_iter(self) -> HtmlCollectionIter {
        HtmlCollectionIter {
            values: self,
            index: 0,
        }
    }
}

pub struct HtmlCollectionIter {
    values: HtmlCollection,
    index: u32,
}

impl Iterator for HtmlCollectionIter {
    type Item = web_sys::Element;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.values.length() {
            let value = self.values.item(self.index).unwrap();
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

// Check if there are events on the specified day of the current week
pub fn has_event_on_day(events: &Vec<RawEvent>, current_day: NaiveDateTime, day_to_look: Weekday) -> bool {
    let offset_to_saturday = day_to_look.num_days_from_monday() as i64 - current_day.weekday().number_from_monday() as i64 + 1;
    let saturday_ts = (current_day + chrono::Duration::days(offset_to_saturday)).timestamp() as u64;
    let range = saturday_ts..saturday_ts + 3600*24;
    if events.is_empty() { return false }
    match events.binary_search_by(|e| e.start_unixtime.cmp(&saturday_ts)) {
        Ok(_) => true,
        Err(i) => {
            if i < events.len() {
                range.contains(&events[i].start_unixtime)
            } else {
                false
            }
        },
    }
}
