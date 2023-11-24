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
