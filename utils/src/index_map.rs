use crate::FastHashMap;

pub struct IndexMap<V> {
    vec: Vec<(String, V)>,
    map: FastHashMap<String, usize>,
}

impl<V> Default for IndexMap<V> {
    fn default() -> Self {
        Self {
            vec: Vec::new(),
            map: FastHashMap::default(),
        }
    }
}

impl<V> IndexMap<V> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            map: FastHashMap::default(),
        }
    }

    pub fn insert(&mut self, key: String, value: V) {
        if let Some(&idx) = self.map.get(&key) {
            self.vec[idx].1 = value;
        } else {
            let idx = self.vec.len();

            self.vec.push((key.clone(), value));
            self.map.insert(key, idx);
        }
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        let &idx = self.map.get(key)?;
        Some(&self.vec[idx].1)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        let &idx = self.map.get(key)?;
        Some(&mut self.vec[idx].1)
    }

    pub fn remove(&mut self, key: &str) -> Option<V> {
        let idx = self.map.remove(key)?;
        let (_, value) = self.vec.remove(idx);

        for (k, _) in &self.vec[idx..] {
            *self.map.get_mut(k).unwrap() -= 1;
        }

        Some(value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.vec.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut V)> {
        self.vec.iter_mut().map(|(k, v)| (k.as_str(), v))
    }
}
