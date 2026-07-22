use crate::shared::eviction_policy::EvictionPolicyKind;
use log::error;

#[derive(Debug, Clone, Copy)]
pub struct AssociativeCacheSettings {
	pub num_of_buckets: usize,
	pub bucket_size: usize,
	pub eviction_policy: EvictionPolicyKind,
}

impl AssociativeCacheSettings {
	/// Total slots the layout holds: the budget the geometry seam must conserve.
	pub fn capacity(&self) -> usize {
		self.num_of_buckets * self.bucket_size
	}

	/// Replace the bucket layout with a geometry-resolved split, leaving the
	/// eviction policy untouched.
	pub fn with_geometry(&self, num_of_buckets: usize, bucket_size: usize) -> Self {
		AssociativeCacheSettings { num_of_buckets, bucket_size, ..*self }
	}

	pub fn validate(&self) -> Result<(), Vec<String>> {
		let mut problems = Vec::new();
		if self.num_of_buckets == 0 {
			problems.push("num_of_buckets must be > 0".to_string());
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

	pub fn from_jni_args(
		num_of_buckets: i64,
		bucket_size: i64,
		eviction_policy: i64,
	) -> Option<Self> {
		if eviction_policy < 0 {
			error!("AssociativeCacheSettings.from_jni_args: invalid eviction_policy={eviction_policy}");
			return None;
		}
		let policy = match EvictionPolicyKind::from_usize(eviction_policy as usize) {
			Some(p) => p,
			None => {
				error!("AssociativeCacheSettings.from_jni_args: unknown eviction_policy={eviction_policy}");
				return None;
			}
		};
		let settings = AssociativeCacheSettings {
			num_of_buckets: num_of_buckets.max(0) as usize,
			bucket_size: bucket_size.max(0) as usize,
			eviction_policy: policy,
		};
		if let Err(problems) = settings.validate() {
			error!("AssociativeCacheSettings.from_jni_args: invalid settings: {}", problems.join(", "));
			return None;
		}
		Some(settings)
	}
}

impl Default for AssociativeCacheSettings {
	fn default() -> Self {
		AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: 8,
			eviction_policy: EvictionPolicyKind::Lfu,
		}
	}
}
