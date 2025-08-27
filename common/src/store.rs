use crate::prelude::*;

type CrdtValue<T> = (Option<T>, i64);
type CrdtBool = (bool, i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtMap<K: Eq+std::hash::Hash, V>(HashMap<K, CrdtValue<V>>);

impl<K: Eq+std::hash::Hash, V> CrdtMap<K, V> {
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtSet<K: Eq+std::hash::Hash>(HashMap<K, CrdtBool>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtStore {
    pub created_ts: i64,
    pub colors: CrdtMap<String, String>,
    pub seen_announcements: CrdtMap<String, String>,
    pub seen_homeworks: CrdtSet<String>,
    pub hidden_courses: CrdtSet<String>,
}

impl CrdtStore {
    /// When merging stores, it doesn't only compute the union of the sets.
    /// Otherwise, it would become impossible to unhide a course.
    /// Instead, 
    pub fn merge(self, other: CrdtStore) -> CrdtStore {
        todo!("Implement Store::merge")
    }
}
