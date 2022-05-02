use std::{collections::HashMap, sync::{Mutex, atomic::AtomicBool}};
use crate::prelude::*;

lazy_static::lazy_static!{
    pub static ref COLORS: Colors = Colors::restore();
    pub static ref COLORS_CHANGED: AtomicBool = AtomicBool::new(false);
}

pub struct Colors {
    light: Mutex<HashMap<String, (String, String)>>,
    dark: Mutex<HashMap<String, (String, String)>>,
}

impl Colors {
    fn restore() -> Colors {
        let local_storage = window().local_storage().unwrap().unwrap();
        let light = match local_storage.get_item("light-colors").unwrap() {
            Some(json) => serde_json::from_str(&json).unwrap_or_default(),
            None => HashMap::new(),
        };
        let dark = match local_storage.get_item("dark-colors").unwrap() {
            Some(json) => serde_json::from_str(&json).unwrap_or_default(),
            None => HashMap::new(),
        };

        Colors {
            light: Mutex::new(light),
            dark: Mutex::new(dark),
        }
    }

    pub fn get(&self, course: &str) -> (String, String) {
        let inner = match crate::settings::SETTINGS.theme() {
            crate::settings::Theme::Light => self.light.lock().unwrap(),
            crate::settings::Theme::Dark => self.dark.lock().unwrap(),
            crate::settings::Theme::System => self.dark.lock().unwrap(), // Remove themes colors kits

        };
        inner.get(course).map(|(v,w)| (v.to_string(),w.to_string())).unwrap_or_else(|| (String::from("#98fb98"), String::from("black")))
    }

    pub fn set(&self, course: &str, background_color: String, text_color: String) {
        let mut inner = match crate::settings::SETTINGS.theme() {
            crate::settings::Theme::Light => self.light.lock().unwrap(),
            crate::settings::Theme::Dark => self.dark.lock().unwrap(),
            crate::settings::Theme::System => todo!("System colors"),
        };
        inner.insert(course.to_string(), (background_color, text_color));
        std::mem::drop(inner);
        self.save();
    }

    fn save(&self) {
        let local_storage = window().local_storage().unwrap().unwrap();
        local_storage.set_item("light-colors", &serde_json::to_string(&self.light).unwrap()).unwrap();
        local_storage.set_item("dark-colors", &serde_json::to_string(&self.dark).unwrap()).unwrap();
    }
}
