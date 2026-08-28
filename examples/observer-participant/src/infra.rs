//! The infra layer, which `api` is supposed to reach only through a seam.

/// A trivial store, standing in for whatever infrastructure a real crate keeps here.
#[derive(Default)]
pub struct Store {
    entries: Vec<(String, String)>,
}

impl Store {
    /// Insert a pair.
    pub fn insert(&mut self, key: &str, value: &str) {
        self.entries.push((key.to_string(), value.to_string()));
    }

    /// The value for `key`, if present.
    pub fn get(&self, key: &str) -> Option<String> {
        self.entries
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value.clone())
    }
}
