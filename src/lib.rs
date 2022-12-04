use std::{
    borrow::Borrow,
    collections::hash_map::DefaultHasher,
    fmt::Display,
    hash::{Hash, Hasher},
    mem,
    ops::Index,
};

const INITAL_NBUCKETS: usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }
        let position = self.position(&key);
        let bucket = &mut self.buckets[position];

        self.items += 1;
        for (ekey, evalue) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }
        bucket.push((key, value));
        None
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let position = self.position(key);
        self.buckets[position]
            .iter()
            .find(|&(ref ekey, _)| ekey.borrow() == key)
            .map(|&(_, ref v)| v)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let position = self.position(key);
        let bucket = &mut self.buckets[position];
        let i = bucket
            .iter()
            .position(|&(ref ekey, _)| ekey.borrow() == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(i).1)
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    fn position<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITAL_NBUCKETS,
            n => n * 2,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));
        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let position = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[position].push((key, value));
        }

        let _ = mem::replace(&mut self.buckets, new_buckets);
    }
}

pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => {
                    match bucket.get(self.at) {
                        Some(&(ref k, ref v)) => {
                            // move along self.at and self.bucket
                            self.at += 1;
                            break Some((k, v));
                        }
                        None => {
                            self.bucket += 1;
                            self.at = 0;
                            continue;
                        }
                    }
                }
                None => break None,
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            bucket: 0,
            at: 0,
        }
    }
}

impl<K, Q, V> Index<&Q> for HashMap<K, V>
where
    K: Borrow<Q>,
    K: Hash + Eq,
    Q: Hash + Eq + ?Sized + Display,
{
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect(&format!("key `{}` not found", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        map.insert("abc", 123);
        assert_eq!(map.get("abc"), Some(&123));
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
        map.remove("abc");
        assert_eq!(map.get("abc"), None);
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn iter() {
        let mut map = HashMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);
        map.insert("d", 4);

        for (&k, &v) in &map {
            match k {
                "a" => assert_eq!(v, 1),
                "b" => assert_eq!(v, 2),
                "c" => assert_eq!(v, 3),
                "d" => assert_eq!(v, 4),
                _ => unreachable!(),
            }
        }

        assert_eq!((&map).into_iter().count(), 4);
    }
}
