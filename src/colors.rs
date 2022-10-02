use std::{collections::HashMap, sync::{Mutex, atomic::AtomicBool, Arc}};
use crate::prelude::*;

lazy_static::lazy_static!{
    pub static ref COLORS: Arc<Colors> = Arc::new(Colors::restore());
    pub static ref COLORS_CHANGED: AtomicBool = AtomicBool::new(false);
}

pub struct Colors {
    content: Mutex<HashMap<String, String>>,
    to_publish: Arc<Mutex<Vec<(String, String)>>>,
}

impl Colors {
    fn restore() -> Colors {
        let local_storage = window().local_storage().unwrap().unwrap();
        
        // Convert new color's system  
        let tmp_colors = local_storage.get_item("colors").unwrap();
        let colors = match tmp_colors {
            Some(json) => serde_json::from_str(&json).unwrap_or_default(),
            None => HashMap::new(),
        };

        Colors {
            content: Mutex::new(colors),
            to_publish: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get(&self, course: &str) -> String {
        match self.content.try_lock() {
            Ok(v) => v.get(course).map(|v| v.to_string()).unwrap_or_else(|| String::from("#CB6CE6")),
            Err(_) => {sentry_report(JsValue::from_str("try lock impossible")); String::from("#CB6CE6")},
        }
    }

    pub fn set(&self, course: &str, background_color: String) {
        match self.content.try_lock(){
            Ok(mut v) => {
                v.insert(course.to_string(), background_color.clone());
            },
            Err(_) => sentry_report(JsValue::from_str("try lock impossible")),
        }
        match self.to_publish.as_ref().try_lock() {
            Ok(mut v) => v.push((course.to_string(), background_color)),
            Err(_) => sentry_report(JsValue::from_str("try lock impossible")),
        } 
        self.save();
    }

    fn save(&self) {
        let local_storage = window().local_storage().unwrap().unwrap();
        local_storage.set_item("colors", &serde_json::to_string(&self.content).unwrap()).unwrap();
    }

    pub fn fetch_colors(&self, ctx: &Context<App>) {
        let local_storage = window().local_storage().unwrap().unwrap();

        if let Some(time) = local_storage.get_item("last_colors_updated").unwrap() {
            let last_updated = time.parse::<i64>().unwrap();
            let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;
            if now - last_updated < 15 { 
                return;
            }   
        }

        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::get_colors().await  {
                Ok(events) => link.send_message(AppMsg::FetchColors(events)),
                Err(e) => link.send_message(AppMsg::ApiFailure(e)),
            }
        });
    }

    pub fn update_colors(&self, colors: HashMap<String, String>) {
        // Merge new colors
        let mut content = match self.content.try_lock() {
            Ok(v) => v,
            Err(_) => {sentry_report(JsValue::from_str("try lock impossible")); return},
        };
        let mut to_publish = match self.to_publish.as_ref().try_lock() {
            Ok(v) => v,
            Err(_) => {sentry_report(JsValue::from_str("try lock impossible")); return},
        };
        for (course, color) in content.iter() {
            if !colors.contains_key(course) {
                to_publish.push((course.to_string(), color.to_string()));
            }
        }
        content.extend(colors);
        // Save last updated time
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;

        let local_storage = window().local_storage().unwrap().unwrap();
        local_storage.set_item("last_colors_updated", &now.to_string()).unwrap();
        drop(content);
        crate::colors::COLORS_CHANGED.store(true, std::sync::atomic::Ordering::Relaxed);
        self.save();
    }

    pub fn push_colors(&self) {
        let to_publish2 = Arc::clone(&self.to_publish);
        wasm_bindgen_futures::spawn_local(async move {
            let mut to_publish2 = match to_publish2.as_ref().try_lock() {
                Ok(v) => v,
                Err(_) => {sentry_report(JsValue::from_str("try lock impossible")); return},
            };

            if !to_publish2.is_empty() && crate::api::publish_colors(&to_publish2.clone()).await.is_ok() {
                to_publish2.clear();
            }
        });
    }
}
