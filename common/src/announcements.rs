use serde::{Serialize, Deserialize};
use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentType {
    Text,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnnouncementDesc {
    pub title: String,
    pub id: String,
    pub start_ts: u64,
    pub end_ts: u64,
    pub target: Option<Groups>,
    pub max_impressions: Option<u64>,
    pub closable: bool,
    pub ty: ContentType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_fr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_en: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
}
