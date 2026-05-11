use crate::shared::eviction_policy::EvictionPolicy;
use log::error;

#[derive(Debug, Clone, Copy)]
pub struct DashSettings {
	pub num_of_segments: usize,
	pub num_of_normal_buckets: usize,
	pub num_of_stash_buckets: usize,
	pub bucket_size: usize,
	pub eviction_policy: EvictionPolicy,
	pub debug_mode: usize,
}

impl Default for DashSettings {
	fn default() -> Self {
		DashSettings {
			num_of_segments: 1,
			num_of_normal_buckets: 28,
			num_of_stash_buckets: 4,
			bucket_size: 16,
			eviction_policy: EvictionPolicy::ClassicLRU,
			debug_mode: 1,
		}
	}
}

impl DashSettings {
	pub fn from_jni_args(
		num_of_segments: i64,
		num_of_normal_buckets: i64,
		num_of_stash_buckets: i64,
		bucket_size: i64,
		eviction_policy: i64,
		debug_mode: i64,
	) -> Option<Self> {
		let mut problems: Vec<String> = Vec::new();
		if num_of_segments <= 0 { problems.push(format!("num_of_segments={num_of_segments}")); }
		if num_of_normal_buckets <= 0 { problems.push(format!("num_of_normal_buckets={num_of_normal_buckets}")); }
		if num_of_stash_buckets < 0 { problems.push(format!("num_of_stash_buckets={num_of_stash_buckets}")); }
		if bucket_size <= 0 { problems.push(format!("bucket_size={bucket_size}")); }
		if eviction_policy < 0 { problems.push(format!("eviction_policy={eviction_policy}")); }
		if debug_mode < 0 { problems.push(format!("debug_mode={debug_mode}")); }
		if !problems.is_empty() {
			error!("DashSettings.from_jni_args: invalid args: {}", problems.join(", "));
			return None;
		}

		Some(DashSettings {
			num_of_segments: num_of_segments as usize,
			num_of_normal_buckets: num_of_normal_buckets as usize,
			num_of_stash_buckets: num_of_stash_buckets as usize,
			bucket_size: bucket_size as usize,
			eviction_policy: EvictionPolicy::from_usize(eviction_policy as usize)?,
			debug_mode: debug_mode as usize,
		})
	}
}

#[allow(dead_code)] // Remove this after stabilizing hit ratio
pub const DRAGON_SETTINGS: DashSettings = DashSettings {
	num_of_segments: 1,
	num_of_normal_buckets: 56,
	num_of_stash_buckets: 4,
	bucket_size: 14,
	eviction_policy: EvictionPolicy::ClassicLRU,
	debug_mode: 1,
};
