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
	/// Construct from i64 parameters. Returns None on invalid values.
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

#[cfg(test)]
mod tests {
	use super::*;

	fn valid_args() -> [i64; 6] {
		// (segments, normal_buckets, stash_buckets, bucket_size, policy, debug_mode)
		[1, 4, 2, 2, 0, 0]
	}

	fn call(args: [i64; 6]) -> Option<DashSettings> {
		DashSettings::from_jni_args(args[0], args[1], args[2], args[3], args[4], args[5])
	}

	#[test]
	fn from_jni_args_accepts_valid() {
		let s = call(valid_args()).expect("valid args must parse");
		assert_eq!(s.num_of_segments, 1);
		assert_eq!(s.num_of_normal_buckets, 4);
		assert_eq!(s.num_of_stash_buckets, 2);
		assert_eq!(s.bucket_size, 2);
		assert!(matches!(s.eviction_policy, EvictionPolicy::ClassicLRU));
		assert_eq!(s.debug_mode, 0);
	}

	#[test]
	fn from_jni_args_rejects_invalid_policy_ordinal() {
		let mut args = valid_args();
		args[4] = 99; // out-of-range policy ordinal
		assert!(call(args).is_none(), "invalid policy ordinal must return None");
	}

	#[test]
	fn from_jni_args_rejects_non_positive_segments() {
		let mut args = valid_args();
		args[0] = 0;
		assert!(call(args).is_none(), "num_of_segments must be > 0");
		args[0] = -1;
		assert!(call(args).is_none(), "num_of_segments must be > 0");
	}

	#[test]
	fn from_jni_args_rejects_non_positive_normal_buckets() {
		let mut args = valid_args();
		args[1] = 0;
		assert!(call(args).is_none(), "num_of_normal_buckets must be > 0");
		args[1] = -3;
		assert!(call(args).is_none(), "num_of_normal_buckets must be > 0");
	}

	#[test]
	fn from_jni_args_rejects_negative_stash_buckets() {
		let mut args = valid_args();
		args[2] = -1;
		assert!(call(args).is_none(), "num_of_stash_buckets must be >= 0");
	}

	#[test]
	fn from_jni_args_rejects_non_positive_bucket_size() {
		let mut args = valid_args();
		args[3] = 0;
		assert!(call(args).is_none(), "bucket_size must be > 0");
		args[3] = -5;
		assert!(call(args).is_none(), "bucket_size must be > 0");
	}

	#[test]
	fn from_jni_args_rejects_negative_policy() {
		let mut args = valid_args();
		args[4] = -1;
		assert!(call(args).is_none(), "eviction_policy must be >= 0");
	}

	#[test]
	fn from_jni_args_rejects_negative_debug_mode() {
		let mut args = valid_args();
		args[5] = -1;
		assert!(call(args).is_none(), "debug_mode must be >= 0");
	}
}
