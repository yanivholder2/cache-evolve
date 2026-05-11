pub mod dash_combined_jni;
pub mod dash_promotion_cold_jni;
pub mod dash_promotion_hot_jni;

use crate::dash::dash_settings::DashSettings;
use crate::shared::logging;
use jni::sys::jlong;
use log::error;

/// Initializes the file logger early so subsequent `error!` calls land in `rust-logs/`
/// (not stderr, which Caffeine's simulator swallows), then parses the six JNI
/// arguments into a `DashSettings`. Returns `None` on bad args after logging which
/// argument failed; the JNI caller should return `0` (the null-cache sentinel).
pub(crate) fn from_jni_args_or_log(
	cache_name: &str,
	num_of_segments: jlong,
	num_of_normal_buckets: jlong,
	num_of_stash_buckets: jlong,
	bucket_size: jlong,
	eviction_policy: jlong,
	debug_mode: jlong,
) -> Option<DashSettings> {
	// `debug_mode` may be invalid; clamp to a safe default just for the early logger so
	// init failures still land in the log file. `init_file_logger` is idempotent.
	let early_debug_mode = if (0..=4).contains(&debug_mode) { debug_mode as usize } else { 1 };
	logging::init_file_logger(cache_name, early_debug_mode);

	match DashSettings::from_jni_args(
		num_of_segments, num_of_normal_buckets, num_of_stash_buckets,
		bucket_size, eviction_policy, debug_mode,
	) {
		Some(settings) => Some(settings),
		None => {
			error!("{cache_name}.init: rejecting invalid args: segments={num_of_segments} normal_buckets={num_of_normal_buckets} stash={num_of_stash_buckets} bucket_size={bucket_size} policy={eviction_policy} debug={debug_mode}");
			None
		}
	}
}
