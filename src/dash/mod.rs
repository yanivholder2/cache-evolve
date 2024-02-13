use std::fmt::{Display, Formatter};
use std::hash::Hash;

use crate::dash::dash_segment::DashSegment;
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::utils::{get_index, hash};

mod dash_bucket;
pub mod dash_jni;
pub(crate) mod dash_segment;
mod dash_settings;

#[derive(Debug)]
pub struct Dash<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	pub segments: Vec<DashSegment<K, V>>,
}

impl<K, V> Dash<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	/// Creates a new Dash instance with the given settings.
	///
	/// ### Arguments
	/// * `settings` - The settings for the Dash instance.
	///
	/// ### Example
	/// ```rust
	/// use dash::Dash;
	///
	/// let settings = DashSettings {
	///  dash_size: 1,
	///  segment_size: 2,
	///  stash_size: 1,
	///  bucket_size: 3,
	///  eviction_policy: EvictionPolicy::Lru,
	///  debug_mode: 0,
	/// };
	///
	/// let dash = Dash::new(settings);
	/// ```
	///
	/// ### Returns
	/// Returns a new Dash instance.
	///
	pub fn new(settings: DashSettings) -> Self {
		// TODO: think about maybe using Vec::with_capacity
		let mut segments = Vec::new();
		for _ in 0..settings.dash_size {
			// TODO: pass the settings as a reference
			segments.push(DashSegment::new(settings.clone()));
		}
		Self { segments }
	}

	/// Insert a key-value pair into Dash
	pub fn put(&mut self, key: K, value: V) {
		let segment = self.get_mut_segment(&key);
		segment.put(Item::new(key, value));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	/// As a side effect if this key is already exists with a
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let segment = self.get_mut_segment(key);
		let data = segment.get(key)?;
		Some(&data.value)
	}

	fn get_mut_segment(&mut self, key: &K) -> &mut DashSegment<K, V> {
		let hash = hash(key);
		// TODO: change .len to .size
		let segment_index = get_index(hash, self.segments.len());
		&mut self.segments[segment_index]
	}
}

impl<K, V> Display for Dash<K, V>
where
	K: Hash + Eq + Copy + Display,
	V: Eq + Copy + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for segment in &self.segments {
			writeln!(f, "{}", segment)?;
		}
		Ok(())
	}
}

/*
#[cfg(test)]
mod tests {

	use super::*;
	use crate::dash_settings::{EvictionPolicy, DEFAULT_SETTINGS};

	fn record(dash: &mut Dash<i64, i64>, key: i64, value: i64) {
		let res = dash.get_and_update_item(&key);
		if res.is_none() {
			dash.put(key, value);
		}
	}

	#[test]
	fn get_without_put() {
		let mut dash: Dash<i64, i64> = Dash::new(DEFAULT_SETTINGS);
		let key: i64 = 0;

		assert_eq!(dash.get_and_update_item(&key), None);
	}

	#[test]
	fn get_after_different_value_put() {
		let mut dash: Dash<i64, i64> = Dash::new(DEFAULT_SETTINGS);
		let key: i64 = 0;

		dash.put(key, key);

		assert_eq!(dash.get_and_update_item(&key), None);
	}

	#[test]
	fn get_after_same_value_put() {
		let mut dash: Dash<i64, i64> = Dash::new(DEFAULT_SETTINGS);
		let key: i64 = 0;

		dash.put(key, key);

		assert_eq!(dash.get_and_update_item(&key), Some(&key));
	}

	#[test]
	fn print_dash() {
		let mut dash: Dash<i64, i64> = Dash::new(DEFAULT_SETTINGS);
		let key: i64 = 0;

		dash.put(key, key);

		println!("{dash}");
	}

	#[test]
	fn big_test() {
		let mut dash: Dash<i64, i64> = Dash::new(DashSettings {
			dash_size: 1,
			segment_size: 2,
			stash_size: 1,
			bucket_size: 3,
			eviction_policy: EvictionPolicy::Lru,
			debug_mode: 0,
		});

		// put in dash random values
		for _ in 0..40 {
			// make a random number from 1 to 10
			let random_number = rand::random::<i64>().abs() % 10 + 1;
			println!("############# {random_number} #############");
			record(&mut dash, random_number, random_number);
			println!("{dash}");
		}
	}
}
*/
