use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::debug;

pub fn get_index(hash: usize, size: usize) -> usize {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_index() {
		let hash = 123;
		let size = 10;
		let index = get_index(hash, size);
		// 123 % 10 = 3
		assert_eq!(index, 3);
	}
}
