use super::associative_cache_settings::AssociativeCacheSettings;
use super::AssociativeCache;
use crate::eviction_policy::EvictionPolicy;

use jni::{objects::JClass, sys::jlong, JNIEnv};

type AssociativeCacheTy = AssociativeCache<i64, i64>;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_initCache<
	'local,
>(
	_env: JNIEnv,
	_class: JClass,
	num_of_buckets: jlong,
	bucket_size: jlong,
	eviction_policy: jlong,
) -> jlong {
	let settings = AssociativeCacheSettings {
		num_of_buckets: num_of_buckets as usize,
		bucket_size: bucket_size as usize,
		eviction_policy: EvictionPolicy::from_usize(eviction_policy as usize).unwrap(),
	};

	let cache: AssociativeCacheTy = AssociativeCache::new(settings);
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;

	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_getFromCacheIfPresent<
	'local,
>(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	let cache = unsafe { &mut *(cache_ptr as *mut AssociativeCacheTy) };
	let res = cache.get_and_update_item(&key);
	match res {
		None => -1,
		Some(value) => *value,
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_putToCache<
	'local,
>(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	let cache = unsafe { &mut *(cache_ptr as *mut AssociativeCacheTy) };
	cache.put(key as i64, value as i64);
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_dropCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
) {
	unsafe { Box::from_raw(cache_ptr as *mut AssociativeCacheTy) };
}
