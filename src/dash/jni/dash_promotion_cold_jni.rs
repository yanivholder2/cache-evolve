use crate::dash::dash_settings::DashSettings;
use crate::dash::segments::PromotionColdSegment;
use crate::dash::Dash;
use crate::shared::logging;

use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::{error, info, trace};

type CacheTy = Dash<i64, i64, PromotionColdSegment<i64, i64>>;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashPromotionColdRustPolicy_initDefaultCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
) -> jlong {
	create_cache(DashSettings::default())
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashPromotionColdRustPolicy_initCache<
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
) -> jlong {
	let Some(settings) = super::from_jni_args_or_log(
		"dash_promotion_cold",
		num_of_segments, num_of_normal_buckets, num_of_stash_buckets,
		bucket_size, eviction_policy, debug_mode,
	) else { return 0; };

	create_cache(settings)
}

fn create_cache(settings: DashSettings) -> jlong {
	if !matches!(settings.eviction_policy, crate::shared::eviction_policy::EvictionPolicy::ClassicLRU) {
		error!("dash_promotion_cold.create_cache: requires ClassicLRU, got {:?}", settings.eviction_policy);
		return 0;
	}
	logging::init_file_logger("dash_promotion_cold", settings.debug_mode);

	let cache: CacheTy = Dash::new(settings.num_of_segments, || {
		PromotionColdSegment::new(&settings)
	});
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;

	info!("create_cache: ptr={cache_ptr} settings={settings:?}");
	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashPromotionColdRustPolicy_getFromCacheIfPresent<
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
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashPromotionColdRustPolicy_putToCache<
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
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashPromotionColdRustPolicy_dropCache<
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
