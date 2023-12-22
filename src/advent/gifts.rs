use std::collections::HashSet;
use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Gift {
    pub day: u8,
    pub kind: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GiftList {
    pub gifts: Vec<Gift>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct CollectedGifts {
    collected: HashSet<u8>
}

impl GiftList {
    pub fn from_json(json: &str) -> Option<Self> {
        let mut gifts: Vec<Gift> = serde_json::from_str(json).ok()?;
        gifts.sort_by_key(|gift| gift.day);
        Some(Self { gifts })
    }

    pub fn get_gift(&self, day: u8) -> Option<Gift> {
        self.gifts.get(day as usize - 1).cloned()
    }

    pub fn get_all_days(&self) -> HashSet<u8> {
        self.gifts.iter().map(|gift| gift.day).collect()
    }
}

impl CollectedGifts {
    pub fn from_local_storage() -> Self {
        let local_storage = window().local_storage().unwrap().unwrap();
        
        match local_storage.get_item("collected_gifts").unwrap() {
            Some(json) => Self::from_json(&json).unwrap_or_default(),
            None => Self::default(),
        }
    }
    
    pub fn from_json(json: &str) -> Option<Self> {
        let collected: HashSet<u8> = serde_json::from_str(json).ok()?;
        Some(Self { collected })
    }

    pub fn get_all_days(&self) -> HashSet<u8> {
        self.collected.clone()
    }

    pub fn is_collected(&self, day: i32) -> bool {
        if !(0..=23).contains(&day) {
            return false;
        }
        let day = day as u8;
        self.collected.contains(&day)
    }

    pub fn collect(&mut self, day: u8) {
        if day == 21 {
            let _ = window().location().reload();
        }
        self.collected.insert(day);
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.collected).unwrap()
    }
}
