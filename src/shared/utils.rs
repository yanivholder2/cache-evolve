use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::debug;

pub fn get_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	let hash = hash(key);
	hash % size
}

pub fn hash<K>(key: &K) -> usize
where
	K: Hash + Debug,
{
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	let hash = hasher.finish() as usize;
	debug!("Hashed key {:?} to {}", key, hash);
	hash
}
