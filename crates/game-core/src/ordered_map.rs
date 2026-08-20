use std::collections::HashMap;

use crate::errors::GameResult;

#[derive(Debug)]
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

    pub fn len(&self) -> usize {
        self.map.len()
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

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.keys.iter()
    }

    pub fn iter_values_mut(
        &mut self,
        mut f: impl FnMut(&mut V) -> GameResult<()>,
    ) -> GameResult<()> {
        for key in &self.keys {
            if let Some(value) = self.map.get_mut(key) {
                f(value)?;
            }
        }
        Ok(())
    }
}

impl<K, V> FromIterator<(K, V)> for OrderedMap<K, V>
where
    K: std::hash::Hash + Eq + Clone,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut ordered_map = OrderedMap::new();
        for (key, value) in iter {
            ordered_map.insert(key, value);
        }
        ordered_map
    }
}
