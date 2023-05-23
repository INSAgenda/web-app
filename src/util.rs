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

#[deprecated(note = "Use now() instead")]
pub fn now_ts() -> i64 {
    now()
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
pub fn has_event_on_day(events: &Vec<RawEvent>, current_day: NaiveDate, day_to_look: Weekday) -> bool {
    let offset_to_saturday = day_to_look.num_days_from_monday() as i64 - current_day.weekday().number_from_monday() as i64 + 1;
    let saturday_date = current_day + chrono::Duration::days(offset_to_saturday);
    let saturday_ts = Paris.from_local_datetime(&saturday_date.and_hms_opt(0, 0, 0).unwrap()).unwrap().timestamp() as u64;
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

pub trait HackTraitProfileUrl {
    fn profile_url(&self) -> String;
}

impl HackTraitProfileUrl for UserDesc {
    fn profile_url(&self) -> String {
        format!("https://api.dicebear.com/5.x/identicon/svg?seed={}", self.uid)
    }
}

pub fn now() -> i64 {
    (js_sys::Date::new_0().get_time() / 1000.0) as i64
}

// FIXME: plural forms
pub fn format_time_diff(diff: i64) -> String {
    let words = [["secondes", "minutes", "heures", "jours", "semaines", "mois", "années"], ["seconds ago", "minutes ago", "hours ago", "days ago", "weeks ago", "months ago", "years ago"]];
    let i = usize::from(SETTINGS.lang() != Lang::French);
    if diff < 60 {
        format!("{} {}", diff, words[i][0])
    } else if diff < 3600 {
        format!("{} {}", diff / 60, words[i][1])
    } else if diff < 86400 {
        format!("{} {}", diff / 3600, words[i][2])
    } else if diff < 7*86400 {
        format!("{} {}", diff / 86400, words[i][3])
    } else if diff < 30*86400 {
        format!("{} {}", diff / (7*86400), words[i][4])
    } else if diff < 365*86400 {
        format!("{} {}", diff / (30*86400), words[i][5])
    } else {
        format!("{} {}", diff / (365*86400), words[i][6])
    }
}
