use super::associative_cache_settings::AssociativeCacheSettings;
use super::AssociativeCache;
use crate::shared::eviction_policy::EvictionPolicy;
use crate::shared::logging;

use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::{info, trace};

type AssociativeCacheTy = AssociativeCache<i64, i64>;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_associativeInitCache<
	'local,
>(
	_env: JNIEnv,
	_class: JClass,
	num_of_buckets: jlong,
	bucket_size: jlong,
	eviction_policy: jlong,
	debug_mode: jlong,
) -> jlong {
	let Some(policy) = EvictionPolicy::from_usize(eviction_policy as usize) else {
		eprintln!("ERROR: assoc_jni.init invalid eviction_policy={eviction_policy}");
		return 0;
	};

	let settings = AssociativeCacheSettings {
		num_of_buckets: num_of_buckets as usize,
		bucket_size: bucket_size as usize,
		eviction_policy: policy,
	};

	logging::init_file_logger("associative", debug_mode as usize);
	let cache: AssociativeCacheTy = AssociativeCache::new(settings.clone());
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;
	info!("associativeInitCache: ptr={cache_ptr} settings={settings:?}");
	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_associativeGetFromCacheIfPresent<
	'local,
>(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	if cache_ptr == 0 { return -1; }
	let cache = unsafe { &mut *(cache_ptr as *mut AssociativeCacheTy) };
	let res = cache.get_and_update_item(&key);
	match res {
		None => {
			trace!("associativeGetFromCacheIfPresent: ptr={cache_ptr} key={key} miss");
			-1
		}
		Some(value) => {
			trace!("associativeGetFromCacheIfPresent: ptr={cache_ptr} key={key} hit");
			*value
		}
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_associativePutToCache<
	'local,
>(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	if cache_ptr == 0 { return; }
	let cache = unsafe { &mut *(cache_ptr as *mut AssociativeCacheTy) };
	cache.put(key as i64, value as i64);
	trace!("associativePutToCache: ptr={cache_ptr} key={key} value={value}");
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_associativeDropCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
) {
	if cache_ptr == 0 { return; }
	info!("associativeDropCache: ptr={cache_ptr}");
	unsafe { drop(Box::from_raw(cache_ptr as *mut AssociativeCacheTy)) };
}
