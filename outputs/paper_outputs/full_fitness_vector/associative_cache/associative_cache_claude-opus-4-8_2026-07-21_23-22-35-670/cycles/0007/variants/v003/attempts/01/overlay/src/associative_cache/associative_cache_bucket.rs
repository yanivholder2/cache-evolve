use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Segmented-LRU bucket (H03).
///
/// The single ClassicLRU list is replaced by a two-segment ordering. Orientation:
/// index 0 is the eviction end; higher indices are more protected. The protected
/// segment occupies the top `protected_len` indices `[len - protected_len, len)`;
/// the probationary segment is `[0, len - protected_len)`.
///
/// `protected_len` lives off-`Item` on the bucket struct, so the per-item metadata
/// floor (key + value) is unchanged. All relocations are single-element rotations,
/// so per-op cost stays bounded by `bucket_size`.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	/// Number of items in the protected segment (the top `protected_len` indices).
	protected_len: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			protected_len: 0,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	/// Cap on the protected segment: `bucket_size - 1` (reserve exactly one
	/// probationary slot).
	///
	/// Cycle 6 proved `max_size / 2` demotes genuine reuse winners back into
	/// scan-fodder and dropped hit-rate. Raising the cap to `max_size - 1` lets
	/// protection grow until only a single live probationary slot remains, so a
	/// probationary victim sits at index 0 in the common case while reuse winners
	/// are never demoted prematurely. `saturating_sub(1)` keeps the cap at 0 for a
	/// degenerate one-slot bucket (`max_size == 0` or `1`).
	fn protected_cap(&self) -> usize {
		self.max_size.saturating_sub(1)
	}
}

impl<K, V> Bucket<K, V> for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		&mut self.items
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	/// Front-admission SLRU put.
	///
	/// A miss inserts at index 0 (probationary, at the eviction end); the new item
	/// is probationary so `protected_len` is unchanged. When full, eviction always
	/// removes index 0 — a probationary item, unless the bucket is entirely
	/// protected. An update to an existing key delegates to the hit path.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}

		let evicted = if self.is_full() {
			let removed = self.items.remove(0);
			// Index 0 is protected only when the bucket is entirely protected
			// (protected_len == old len). Keep protected_len within the new length.
			if self.protected_len > self.items.len() {
				self.protected_len = self.items.len();
			}
			Some(removed)
		} else {
			None
		};

		// Front-admission: the fresh item enters at the eviction end.
		self.items.insert(0, item);
		(&self.items[0], evicted)
	}

	/// SLRU access side effect.
	///
	/// - A hit on a protected item moves it to the protected MRU (the back),
	///   leaving `protected_len` unchanged.
	/// - A hit on a probationary item promotes it into the protected segment and
	///   increments `protected_len`; if that exceeds the cap, the protected LRU is
	///   demoted back to probationary MRU (a pure boundary move).
	///
	/// All relocations are single-element rotations.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let len = self.items.len();
		// Start index of the protected segment: protected is [boundary, len).
		let boundary = len - self.protected_len;

		let result_index = if position >= boundary {
			// Already protected: rotate to the very back (protected MRU).
			self.items[position..].rotate_left(1);
			len - 1
		} else {
			// Probationary: promote toward the protected segment.
			// The item rotates up to `end`; when there is no protected segment yet
			// it becomes the sole protected item at the back (len - 1).
			let end = if self.protected_len == 0 { len - 1 } else { boundary };
			self.items[position..=end].rotate_left(1);
			self.protected_len += 1;
			if self.protected_len > self.protected_cap() {
				// Demote the protected LRU back to probationary MRU: a pure boundary
				// move, no data shuffle needed beyond the rotation already done.
				self.protected_len -= 1;
			}
			end
		};

		&self.items[result_index]
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in &self.items {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}
