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
	pub fn validate(&self) -> Result<(), Vec<String>> {
		let mut problems = Vec::new();
		if self.num_of_segments == 0 {
			problems.push("num_of_segments must be > 0".to_string());
		}
		if self.num_of_normal_buckets < 2 {
			problems.push(format!(
				"num_of_normal_buckets={} (must be >= 2 for probing)",
				self.num_of_normal_buckets
			));
		}
		if self.num_of_stash_buckets == 0 {
			problems.push("num_of_stash_buckets must be > 0".to_string());
		}
		if self.bucket_size == 0 {
			problems.push("bucket_size must be > 0".to_string());
		}
		if problems.is_empty() {
			Ok(())
		} else {
			Err(problems)
		}
	}

	/// Construct from i64 parameters. Returns None on invalid values.
	pub fn from_jni_args(
		num_of_segments: i64,
		num_of_normal_buckets: i64,
		num_of_stash_buckets: i64,
		bucket_size: i64,
		eviction_policy: i64,
		debug_mode: i64,
	) -> Option<Self> {
		if eviction_policy < 0 {
			error!("DashSettings.from_jni_args: invalid eviction_policy={eviction_policy}");
			return None;
		}
		if debug_mode < 0 {
			error!("DashSettings.from_jni_args: invalid debug_mode={debug_mode}");
			return None;
		}
		let eviction_policy_val = match EvictionPolicy::from_usize(eviction_policy as usize) {
			Some(p) => p,
			None => {
				error!("DashSettings.from_jni_args: invalid eviction_policy={eviction_policy}");
				return None;
			}
		};
		let settings = DashSettings {
			num_of_segments: num_of_segments.max(0) as usize,
			num_of_normal_buckets: num_of_normal_buckets.max(0) as usize,
			num_of_stash_buckets: num_of_stash_buckets.max(0) as usize,
			bucket_size: bucket_size.max(0) as usize,
			eviction_policy: eviction_policy_val,
			debug_mode: debug_mode as usize,
		};
		if let Err(problems) = settings.validate() {
			error!("DashSettings.from_jni_args: invalid settings: {}", problems.join(", "));
			return None;
		}
		Some(settings)
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_jni_args_rejects_single_normal_bucket() {
		assert!(DashSettings::from_jni_args(1, 1, 2, 16, 0, 0).is_none());
	}

	#[test]
	fn from_jni_args_rejects_zero_stash_buckets() {
		assert!(DashSettings::from_jni_args(1, 4, 0, 16, 0, 0).is_none());
	}
}
