use crate::prelude::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventKind {
    Td,
    Cm,
    Tp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub summary: String,
    pub kind: Option<EventKind>,
    pub number: Option<u8>,
    pub teachers: Vec<String>,
    pub groups: Groups,
    pub location: Option<Location>,
    pub start_unixtime: u64,
    pub end_unixtime: u64,
    #[serde(default)]
    pub eid: String,
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.eid.cmp(&other.eid))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.eid.cmp(&other.eid)
    }
}
