use std::{
	fmt::{Display, Formatter},
	hash::Hash,
};

#[derive(Debug, Clone)]
pub struct Data<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	pub key: K,
	pub value: V,
	pub lfu_counter: usize,
}

impl<K, V> PartialEq for Data<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	fn eq(&self, other: &Data<K, V>) -> bool {
		self.key == other.key && self.value == other.value
	}
}

impl<K, V> Eq for Data<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
}

impl<K, V> Data<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	pub fn new(key: K, value: V) -> Self {
		Self {
			key,
			value,
			lfu_counter: 0,
		}
	}
}

impl<K, V> Display for Data<K, V>
where
	K: Hash + Eq + Clone + Display,
	V: Eq + Clone + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Data {{ key: {}, value: {}, lfu_counter: {} }}",
			self.key, self.value, self.lfu_counter
		)
	}
}
