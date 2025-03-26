use dashmap::DashSet;

pub struct LocalDictionary(DashSet<String>);

// Local dictionary abstraction lowercases all spell checks
impl LocalDictionary {
    pub fn new() -> Self {
        Self(DashSet::new())
    }

    pub fn contains(&self, v: &str) -> bool {
        self.0.contains(&v.to_lowercase())
    }

    pub fn insert(&self, v: String) {
        self.0.insert(v.to_lowercase());
    }
}
