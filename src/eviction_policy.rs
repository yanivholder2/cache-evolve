#[derive(Debug, Clone)]
pub enum EvictionPolicy {
	/** Every cache hit the item will move to the beginning of the data structure */
	ClassicLRU,
	/** LRU will be checked based of timestamp of the last hit and items won't move */
	TimestampLRU,
	Lifo,
	Lfu,
	Fifo,
}

impl EvictionPolicy {
	pub fn from_usize(index: usize) -> Option<Self> {
		match index {
			0 => Some(EvictionPolicy::ClassicLRU),
			1 => Some(EvictionPolicy::Lifo),
			2 => Some(EvictionPolicy::Lfu),
			3 => Some(EvictionPolicy::Fifo),
			4 => Some(EvictionPolicy::TimestampLRU),
			_ => None,
		}
	}
}
