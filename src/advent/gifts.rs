use std::{num::NonZeroU8, collections::HashSet};
use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Gift {
    pub day: u8,
    pub title: String,
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

    pub fn get_gift(&self, day: NonZeroU8) -> Option<Gift> {
        self.gifts.get(day.get() as usize - 1).cloned()
    }

    pub fn get_all_days(&self) -> HashSet<NonZeroU8> {
        self.gifts.iter().map(|gift| NonZeroU8::new(gift.day).unwrap()).collect()
    }
}

impl CollectedGifts {
    pub fn from_json(json: &str) -> Option<Self> {
        let collected: HashSet<u8> = serde_json::from_str(json).ok()?;
        Some(Self { collected })
    }

    pub fn get_all_days(&self) -> HashSet<NonZeroU8> {
        self.collected.iter().map(|day| NonZeroU8::new(*day).unwrap()).collect()
    }

    pub fn is_collected(&self, day: NonZeroU8) -> bool {
        self.collected.contains(&day.get())
    }

    pub fn collect(&mut self, day: NonZeroU8) {
        self.collected.insert(day.get());
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.collected).unwrap()
    }
}
