use dashmap::DashSet;

pub struct LocalDictionary(DashSet<String>);

impl LocalDictionary {
    pub fn new() -> Self {
        Self(DashSet::new())
    }

    pub fn contains(&self, v: &str) -> bool {
        self.0.contains(v)
    }

    pub fn insert(&self, v: String) {
        self.0.insert(v);
    }
}
