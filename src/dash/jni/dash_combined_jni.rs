use crate::dash::dash_settings::DashSettings;
use crate::dash::segments::{CombinedSegment, EvictionTieBreaker};
use crate::dash::Dash;
use crate::shared::eviction_policy::EvictionPolicy;
use crate::shared::logging;

use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::{error, info, trace};

type CacheTy = Dash<i64, i64, CombinedSegment<i64, i64>>;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashCombinedRustPolicy_initDefaultCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
) -> jlong {
	let mut settings = DashSettings::default();
	settings.eviction_policy = EvictionPolicy::TimestampLRU;
	create_cache(settings, EvictionTieBreaker::EvictProbing)
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashCombinedRustPolicy_initCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	num_of_segments: jlong,
	num_of_normal_buckets: jlong,
	num_of_stash_buckets: jlong,
	bucket_size: jlong,
	eviction_policy: jlong,
	debug_mode: jlong,
	tie_breaker: jlong,
) -> jlong {
	let Some(settings) = super::from_jni_args_or_log(
		"dash_combined",
		num_of_segments, num_of_normal_buckets, num_of_stash_buckets,
		bucket_size, eviction_policy, debug_mode,
	) else { return 0; };

	// Wire format: ordinal of EvictionTieBreaker (0 = EvictTarget, 1 = EvictProbing).
	let tie_breaker = match tie_breaker {
		0 => EvictionTieBreaker::EvictTarget,
		1 => EvictionTieBreaker::EvictProbing,
		other => {
			error!("dash_combined.init: invalid tie_breaker ordinal: {other}");
			return 0;
		}
	};
	create_cache(settings, tie_breaker)
}

fn create_cache(settings: DashSettings, tie_breaker: EvictionTieBreaker) -> jlong {
	if !matches!(settings.eviction_policy, EvictionPolicy::TimestampLRU | EvictionPolicy::Lfu) {
		error!("dash_combined.create_cache: requires TimestampLRU or Lfu, got {:?}", settings.eviction_policy);
		return 0;
	}
	logging::init_file_logger("dash_combined", settings.debug_mode);

	let cache: CacheTy = Dash::new(settings.num_of_segments, || {
		CombinedSegment::new(&settings, tie_breaker)
	});
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;

	info!("create_cache: ptr={cache_ptr} settings={settings:?} tie_breaker={tie_breaker:?}");
	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashCombinedRustPolicy_getFromCacheIfPresent<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	if cache_ptr == 0 { return -1; }
	// SAFETY: Java caller guarantees single-threaded access per cache pointer.
	let cache = unsafe { &mut *(cache_ptr as *mut CacheTy) };
	match cache.get_and_update_item(&key) {
		None => {
			trace!("getFromCacheIfPresent: ptr={cache_ptr} key={key} miss");
			-1
		}
		Some(value) => {
			trace!("getFromCacheIfPresent: ptr={cache_ptr} key={key} hit");
			*value
		}
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashCombinedRustPolicy_putToCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	if cache_ptr == 0 { return; }
	// SAFETY: Java caller guarantees single-threaded access per cache pointer.
	let cache = unsafe { &mut *(cache_ptr as *mut CacheTy) };
	cache.put(key, value);
	trace!("putToCache: ptr={cache_ptr} key={key} value={value}");
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashCombinedRustPolicy_dropCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
) {
	if cache_ptr == 0 { return; }
	info!("dropCache: ptr={cache_ptr}");
	// SAFETY: Java caller guarantees single-threaded access and calls dropCache exactly once.
	unsafe { drop(Box::from_raw(cache_ptr as *mut CacheTy)) };
}
