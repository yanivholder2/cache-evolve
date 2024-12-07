use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use crate::shared::item::Item;
use crate::shared::utils::get_index;
use dash_segment::DashSegment;
use dash_settings::DashSettings;
use log::{debug, info};

mod dash_bucket;
pub mod dash_jni;
mod dash_segment;
pub mod dash_settings;

#[derive(Debug)]
pub struct Dash<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub segments: Vec<DashSegment<K, V>>,
}

impl<K, V> Dash<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Creates a new Dash instance with the given settings.
	pub fn new(settings: DashSettings) -> Self {
		info!("Creating a new Dash instance with settings: {:?}", settings);
		// TODO: think about maybe using Vec::with_capacity
		let mut segments = Vec::new();
		for _ in 0..settings.num_of_segments {
			// TODO: pass the settings as a reference
			segments.push(DashSegment::new(settings.clone()));
		}
		Self { segments }
	}

	/// Insert a key-value pair into Dash
	pub fn put(&mut self, key: K, value: V) {
		debug!("Dash: {:?}", self);
		let segment = self.get_mut_segment(&key);
		segment.put(Item::new(key, value));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		debug!("Dash: {:?}", self);
		let segment = self.get_mut_segment(key);
		let data = segment.get(key)?;
		Some(&data.value)
	}

	fn get_mut_segment(&mut self, key: &K) -> &mut DashSegment<K, V> {
		let segment_index = get_index(key, self.segments.len());
		&mut self.segments[segment_index]
	}
}

impl<K, V> Display for Dash<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for segment in &self.segments {
			writeln!(f, "{}", segment)?;
		}
		Ok(())
	}
}
