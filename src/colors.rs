use std::{collections::HashMap, sync::{Mutex, atomic::AtomicBool}};
use crate::prelude::*;

lazy_static::lazy_static!{
    pub static ref COLORS: Colors = Colors::restore();
    pub static ref COLORS_CHANGED: AtomicBool = AtomicBool::new(false);
}

pub struct Colors(Mutex<HashMap<String, String>>);

impl Colors {
    fn restore() -> Colors {
        let local_storage = window().local_storage().unwrap().unwrap();
        
        /* Convert new color's system  */
        let tmp_colors = local_storage.get_item("colors").unwrap();
        let mut colors: HashMap<String, String> = HashMap::new();
        if tmp_colors.is_none(){
            log!("last");
            let light: HashMap<String, (String, String)>  = match local_storage.get_item("light-colors").unwrap() {
                Some(json) => serde_json::from_str(&json).unwrap_or_default(),
                None => HashMap::new(),
            };
            let dark: HashMap<String, (String, String)>  = match local_storage.get_item("dark-colors").unwrap() {
                Some(json) => serde_json::from_str(&json).unwrap_or_default(),
                None => HashMap::new(),
            };
            let mut tmp_colors = HashMap::new();
            /* Collect the colors where user's theme is more filled */
            if light.len() > dark.len(){
                tmp_colors = light;
            }else{
                tmp_colors = dark;
            }
            
            for color in tmp_colors.iter(){
                let (background, _) = color.1;
                colors.insert(color.0.clone(), background.to_string());
            }

            // Save new colors
            local_storage.set_item("colors", &serde_json::to_string(&colors).unwrap()).unwrap();

            // Remove old colors
            local_storage.remove_item("light-colors").unwrap();
            local_storage.remove_item("dark-colors").unwrap();
        }else {
            match tmp_colors {
                Some(json) => colors = serde_json::from_str(&json).unwrap_or_default(),
                None => colors = HashMap::new(),
            }
        }
        
        Colors(Mutex::new(colors))
        
    }

    pub fn get(&self, course: &str) -> String {
        self.0.lock().unwrap().get(course).map(|v| v.to_string()).unwrap_or_else(|| String::from("#CB6CE6"))
    }

    pub fn set(&self, course: &str, background_color: String) {
        self.0.lock().unwrap().insert(course.to_string(), background_color);
        self.save();
    }

    fn save(&self) {
        let local_storage = window().local_storage().unwrap().unwrap();
        local_storage.set_item("colors", &serde_json::to_string(&self.0).unwrap()).unwrap();
    }
}
