use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct VecMap<K, V> where K: Eq {
    vec: Vec<(K, V)>,
}
impl<K, V> Default for VecMap<K, V> where K: Eq {
    fn default() -> Self {
        VecMap::new()
    }
}

impl<K, V> VecMap<K, V> where K: Eq {
    pub fn new() -> Self {
        VecMap { vec: Vec::new() }
    }
    pub fn new_from_vec(vec: Vec<(K, V)>) -> Self {
        let mut out = VecMap::new();
        for (k, v) in vec {
            out.insert(k, v);
        }
        out
    }

    /// returns the old value if the key already exists
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)>{

        if let Some((old_key, old_val)) = self.vec.iter_mut().find(|(k, _)| *k == key) {
            Some((
                std::mem::replace(old_key, key), 
                std::mem::replace(old_val, value)
            ))
        }else{
            self.vec.push((key, value));
            None
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_kvp(key).map(|(_, v)| v)
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.get_kvp_mut(key).map(|(_, v)| v)
    }
    /// # Safety
    /// This function is unsafe because it does not check if the key exists in the map
    /// and will panic if the key does not exist
    pub unsafe fn get_unchecked_mut(&mut self, key: &K) -> &mut V {
        unsafe {
            self.get_unchecked_kvp_mut(key).1
        }
    }

    pub fn get_kvp(&self, key: &K) -> Option<(&K, &V)> {
        for (k, v) in &self.vec {
            if k == key {
                return Some((k, v));
            }
        }
        None
    }

    pub fn get_kvp_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        for (k, v) in &mut self.vec {
            if k == key {
                return Some((k ,v));
            }
        }
        None
    }

    /// # Safety
    /// This function is unsafe because it does not check if the key exists in the map
    /// and will panic if the key does not exist
    pub unsafe fn get_unchecked_kvp_mut(&mut self, key: &K) -> (&K, &mut V) {
        self.get_kvp_mut(key).expect("Key not found")
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.vec.iter().any(|(k, _)| k == key)
    }

    pub fn remove(&mut self, key: &K) -> Option<(K, V)> {
        let mut index = None;
        for (i, (k, _)) in self.vec.iter().enumerate() {
            if k == key {
                index = Some(i);
                break;
            }
        }
        if let Some(i) = index {
            Some(self.vec.remove(i))
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.vec.iter().map(|(k, v)| (k, v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.vec.iter_mut().map(|(k, v)| (k as &K, v))
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }

    pub fn retain<F>(&mut self, mut f: F) where F: FnMut(&K, &V) -> bool {
        self.vec.retain(|(k, v)| f(k, v));
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.vec.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.vec.iter().map(|(_, v)| v)
    }
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.vec.iter_mut().map(|(_, v)| v)
    }

    pub fn as_slice(&self)-> &[(K,V)] {
        self.vec.as_slice()
    }

    pub fn contains(&self, key: &K) -> bool {
        self.vec.iter().any(|(k, _)| k == key)
    }

    pub fn shuffle<R: Rng + ?Sized>(&mut self, rng: &mut R){
        self.vec.shuffle(rng);
    }
    pub fn sort(&mut self)
        where K: Ord + Clone
    {
        self.vec.sort_by_key(|(k, _)|k.clone());
    }
}

impl<K, V> PartialEq for VecMap<K, V> where K: Eq, V: Eq {
    fn eq(&self, other: &Self) -> bool {
        self.vec.iter().all(|(k, v)| other.get(k) == Some(v))
            && other.vec.iter().all(|(k, v)| self.get(k) == Some(v))
    }
}
impl<K, V> Eq for VecMap<K, V> where K: Eq, V: Eq {}


impl<K, V> IntoIterator for VecMap<K, V> where K: Eq {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}
impl<K,  V> FromIterator<(K, V)> for VecMap<K, V> where K: Eq {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut vec = Vec::new();
        for (k, v) in iter {
            vec.push((k, v));
        }
        VecMap { vec }
    }
}

impl<K, V> Serialize for VecMap<K, V> where K: Eq, K: Serialize, V: Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.vec.serialize(serializer)
    }
}
impl<'de, K, V> Deserialize<'de> for VecMap<K, V> where K: Eq, K: Deserialize<'de>, V: Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let vec = Vec::deserialize(deserializer)?;
        Ok(VecMap { vec })
    }
}



pub use macros::vec_map;
mod macros {
    #[macro_export]
    macro_rules! vec_map {
        ($(($key:expr, $value:expr)),*) => {{
            let mut map = $crate::vec_map::VecMap::new();
            $(map.insert($key, $value);)*
            map
        }};
    }

    pub use vec_map;
}