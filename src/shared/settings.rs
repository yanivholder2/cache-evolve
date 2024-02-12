#[derive(Debug, Clone)]
pub enum EvictionPolicy {
	Lru,
	Lifo,
	Lfu,
	Fifo,
}
