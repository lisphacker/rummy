use std::collections::HashMap;

pub struct OrderedMap<K, V> {
    map: HashMap<K, V>,
    keys: Vec<K>,
}

impl<K, V> OrderedMap<K, V>
where
    K: std::hash::Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            keys: Vec::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if !self.map.contains_key(&key) {
            self.keys.push(key.clone());
        }
        self.map.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(value) = self.map.remove(key) {
            self.keys.retain(|k| k != key);
            Some(value)
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys
            .iter()
            .filter_map(move |key| self.map.get(key).map(|value| (key, value)))
    }
}
