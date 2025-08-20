use serde::{Deserialize, Serialize};

use crate::vec_map::VecMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VecSet<K> where K: Eq {
    vec: VecMap<K, ()>,
}
impl<K> Default for VecSet<K> where K: Eq {
    fn default() -> Self {
        VecSet::new()
    }
}

impl <K> VecSet<K> where K: Eq {
    pub fn new() -> Self {
        VecSet { vec: VecMap::new() }
    }

    pub fn insert(&mut self, key: K) -> Option<K> {
        self.vec.insert(key, ()).map(|(k, _)| k)
    }

    pub fn extend(&mut self, keys: impl IntoIterator<Item = K>) {
        for key in keys {
            self.insert(key);
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.vec.get_kvp(key).is_some()
    }

    pub fn remove(&mut self, key: &K) -> Option<K> {
        self.vec.remove(key).map(|(k, _)| k)
    }

    pub fn iter(&self) -> impl Iterator<Item = &K> {
        self.vec.iter().map(|(k, _)| k)
    }

    pub fn count(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }

    pub fn retain<F>(&mut self, mut f: F) where F: FnMut(&K) -> bool {
        self.vec.retain(|k, _| f(k));
    }

    pub fn elements(&self) -> impl Iterator<Item = &K> {
        self.vec.keys()
    }

    pub fn sub(&self, other: &Self) -> Self where K: Clone {
        self.vec.keys().filter(|&k| !other.contains(k)).cloned().collect()
    }
    pub fn intersection(&self, other: &Self) -> Self where K: Clone {
        self.vec.keys().filter(|&k| other.contains(k)).cloned().collect()
    }
    pub fn union(&self, other: &Self) -> Self where K: Clone {
        self.vec.keys().cloned().chain(other.vec.keys().cloned()).collect()
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.vec.keys().all(|k| other.contains(k))
    }
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }
}

impl<K> IntoIterator for VecSet<K> where K: Eq {
    type Item = K;
    type IntoIter = std::iter::Map<std::vec::IntoIter<(K, ())>, fn((K, ())) -> K>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter().map(|(k, _)| k)
    }
}
impl<K> FromIterator<K> for VecSet<K> where K: Eq {
    fn from_iter<I: IntoIterator<Item = K>>(iter: I) -> Self {
        let mut set = VecSet::new();
        for key in iter {
            set.insert(key);
        }
        set
    }
}

impl<K> Serialize for VecSet<K> where K: Eq, K: Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.vec.keys().collect::<Vec<_>>().serialize(serializer)
    }
}
impl<'de, K> Deserialize<'de> for VecSet<K> where K: Eq, K: Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let vec: Vec<K> = Deserialize::deserialize(deserializer)?;
        Ok(vec.into_iter().collect())
    }
}

impl<T: Eq> PartialOrd for VecSet<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Eq> Ord for VecSet<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.count().cmp(&other.count())
    }
}

pub use macros::vec_set;
mod macros {
    #[macro_export]
    macro_rules! vec_set {
        () => {{
            $crate::vec_set::VecSet::new()
        }};
        ($($key:expr),*) => {{
            let mut map = $crate::vec_set::VecSet::new();
            $(map.insert($key);)*
            map
        }};
    }

    pub use vec_set;
}