#[derive(Debug, Clone)]
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
	pub fn from_usize(index: usize) -> Option<Self> {
		match index {
			0 => Some(EvictionPolicy::ClassicLRU),
			1 => Some(EvictionPolicy::TimestampLRU),
			2 => Some(EvictionPolicy::Lifo),
			3 => Some(EvictionPolicy::Lfu),
			4 => Some(EvictionPolicy::Fifo),
			5 => Some(EvictionPolicy::Swap),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_usize_valid_indices() {
		assert!(matches!(EvictionPolicy::from_usize(0), Some(EvictionPolicy::ClassicLRU)));
		assert!(matches!(EvictionPolicy::from_usize(1), Some(EvictionPolicy::TimestampLRU)));
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
}
