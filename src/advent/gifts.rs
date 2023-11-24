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
}
