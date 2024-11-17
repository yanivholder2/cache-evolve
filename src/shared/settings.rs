#[derive(Debug, Clone)]
pub enum EvictionPolicy {
	Lru,
	Lifo,
	Lfu,
	Fifo,
}

impl EvictionPolicy {
	pub fn from_usize(index: usize) -> Option<Self> {
		match index {
			0 => Some(EvictionPolicy::Lru),
			1 => Some(EvictionPolicy::Lifo),
			2 => Some(EvictionPolicy::Lfu),
			3 => Some(EvictionPolicy::Fifo),
			_ => None,
		}
	}
}
