#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
	/** Every cache hit the item will move to the beginning of the data structure */
	ClassicLRU,
	/** LRU will be checked based of timestamp of the last hit and items won't move */
	TimestampLRU,
	Lifo,
	Lfu,
	Fifo,
	Swap,
}

impl EvictionPolicy {
	/// Returns all variants. The exhaustive match ensures a compile error if a variant is added.
	pub fn all() -> [Self; 6] {
		use EvictionPolicy::*;
		let _exhaustive = |p: EvictionPolicy| match p {
			ClassicLRU | TimestampLRU | Lifo | Lfu | Fifo | Swap => {}
		};
		[ClassicLRU, TimestampLRU, Lifo, Lfu, Fifo, Swap]
	}

	/// Canonical CamelCase name. Adding a variant forces a compile error here.
	pub fn name(&self) -> &'static str {
		match self {
			EvictionPolicy::ClassicLRU => "ClassicLRU",
			EvictionPolicy::TimestampLRU => "TimestampLRU",
			EvictionPolicy::Lifo => "Lifo",
			EvictionPolicy::Lfu => "Lfu",
			EvictionPolicy::Fifo => "Fifo",
			EvictionPolicy::Swap => "Swap",
		}
	}

	/// Stable integer index. Adding a variant forces a compile error here.
	pub fn index(&self) -> usize {
		match self {
			EvictionPolicy::ClassicLRU => 0,
			EvictionPolicy::TimestampLRU => 1,
			EvictionPolicy::Lifo => 2,
			EvictionPolicy::Lfu => 3,
			EvictionPolicy::Fifo => 4,
			EvictionPolicy::Swap => 5,
		}
	}

	/// CamelCase names, derived from `all()` + `name()`. Crossed over JNI so
	/// Java callers don't hard-code the list.
	pub fn all_names() -> [&'static str; 6] {
		Self::all().map(|p| p.name())
	}

	pub fn from_usize(index: usize) -> Option<Self> {
		Self::all().into_iter().find(|p| p.index() == index)
	}

	pub fn from_name(name: &str) -> Option<Self> {
		Self::all().into_iter().find(|p| p.name() == name)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_usize_valid_indices() {
		assert!(matches!(
			EvictionPolicy::from_usize(0),
			Some(EvictionPolicy::ClassicLRU)
		));
		assert!(matches!(
			EvictionPolicy::from_usize(1),
			Some(EvictionPolicy::TimestampLRU)
		));
		assert!(matches!(EvictionPolicy::from_usize(2), Some(EvictionPolicy::Lifo)));
		assert!(matches!(EvictionPolicy::from_usize(3), Some(EvictionPolicy::Lfu)));
		assert!(matches!(EvictionPolicy::from_usize(4), Some(EvictionPolicy::Fifo)));
		assert!(matches!(EvictionPolicy::from_usize(5), Some(EvictionPolicy::Swap)));
	}

	#[test]
	fn from_usize_out_of_range() {
		assert!(EvictionPolicy::from_usize(6).is_none());
		assert!(EvictionPolicy::from_usize(100).is_none());
		assert!(EvictionPolicy::from_usize(usize::MAX).is_none());
	}

	#[test]
	fn from_name_valid() {
		assert!(matches!(EvictionPolicy::from_name("ClassicLRU"), Some(EvictionPolicy::ClassicLRU)));
		assert!(matches!(EvictionPolicy::from_name("TimestampLRU"), Some(EvictionPolicy::TimestampLRU)));
		assert!(matches!(EvictionPolicy::from_name("Lifo"), Some(EvictionPolicy::Lifo)));
		assert!(matches!(EvictionPolicy::from_name("Lfu"), Some(EvictionPolicy::Lfu)));
		assert!(matches!(EvictionPolicy::from_name("Fifo"), Some(EvictionPolicy::Fifo)));
		assert!(matches!(EvictionPolicy::from_name("Swap"), Some(EvictionPolicy::Swap)));
	}

	#[test]
	fn from_name_unknown() {
		assert!(EvictionPolicy::from_name("classiclru").is_none());
		assert!(EvictionPolicy::from_name("NotAPolicy").is_none());
		assert!(EvictionPolicy::from_name("").is_none());
	}

	#[test]
	fn name_round_trip_for_every_variant() {
		for p in EvictionPolicy::all() {
			let resolved = EvictionPolicy::from_name(p.name())
				.unwrap_or_else(|| panic!("from_name({}) returned None", p.name()));
			assert_eq!(resolved, p, "{}.name() resolved to wrong variant", p.name());
		}
	}

	#[test]
	fn index_round_trip_for_every_variant() {
		for p in EvictionPolicy::all() {
			let resolved = EvictionPolicy::from_usize(p.index())
				.unwrap_or_else(|| panic!("from_usize({}) returned None", p.index()));
			assert_eq!(resolved, p, "index {} resolved to wrong variant", p.index());
		}
	}

	#[test]
	fn indices_are_dense_and_unique() {
		let indices: Vec<usize> = EvictionPolicy::all().iter().map(|p| p.index()).collect();
		let mut sorted = indices.clone();
		sorted.sort();
		let expected: Vec<usize> = (0..EvictionPolicy::all().len()).collect();
		assert_eq!(sorted, expected, "indices must be 0..N with no gaps or duplicates");
	}

	#[test]
	fn names_are_unique() {
		let names = EvictionPolicy::all_names();
		let mut seen = std::collections::HashSet::new();
		for name in names {
			assert!(seen.insert(name), "duplicate name: {name}");
		}
	}

	#[test]
	fn all_names_matches_per_variant_name() {
		let derived: Vec<&'static str> = EvictionPolicy::all().iter().map(|p| p.name()).collect();
		assert_eq!(derived, EvictionPolicy::all_names().to_vec());
	}
}
