use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Groups {
    pub(crate) groups: HashSet<String>,
}

impl Groups {
    pub fn new() -> Groups {
        Groups { groups: HashSet::new() }
    }

    pub fn new_with_groups(groups: Vec<String>) -> Groups {
        Groups { groups: groups.into_iter().map(|g| g.to_lowercase()).collect() }
    }

    pub fn insert(&mut self, group: &str) {
        self.groups.insert(group.to_lowercase());
    }

    pub fn remove(&mut self, group: &str) {
        self.groups.remove(&group.to_lowercase());
    }

    pub fn matches(&self, another: &Groups) -> bool {
        self.groups.iter().any(|g| another.groups.contains(g))
    }

    pub fn matches_with_name(&self, another: &Groups, _name: Option<&str>) -> bool {
        self.matches(another)
    }

    pub fn read_from_string(s: &str) -> Result<Groups, String> {
        let mut groups = HashSet::new();
        for group in s.split('+') {
            if group.is_empty() {
                continue;
            }
            groups.insert(group.to_lowercase());
        }
        Ok(Groups { groups })
    }

    pub fn format_to_string(&self) -> String {
        let mut groups = self.groups.iter().cloned().collect::<Vec<_>>();
        groups.sort();
        groups.join("+")
    }

    pub fn groups(&self) -> &HashSet<String> {
        &self.groups
    }
}

impl Default for Groups {
    fn default() -> Self {
        Groups {
            groups: [
                "iti3", "stpi2-precedent" , "stpi22-p9-td-01" , "ad-etudiants", "h-22-ang-cult-stpi-gg-01", "h-22-ang-cult-stpi-pg-02", "etudiants", "h-22-com-stpi-08", "stpi21-tp-a1", "stpi22-i4-td-01", "stpi21-all-td-a-j-k", "stpi22-i3-td-01"
            ].iter().map(|g| g.to_string()).collect(),
        }
    }
}

impl Serialize for Groups {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let groups = self.format_to_string();
        serializer.serialize_str(&groups)
    }
}

impl<'de> Deserialize<'de> for Groups {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let groups = String::deserialize(deserializer)?;
        Groups::read_from_string(&groups).map_err(serde::de::Error::custom)
    }
}

impl std::hash::Hash for Groups {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let groups = self.format_to_string();
        groups.hash(state);
    }
}
