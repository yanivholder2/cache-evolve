use super::LruCache;

use jni::{objects::JClass, sys::jlong, JNIEnv};

type LruCacheTy = LruCache<i64, i64>;

const MISS_SENTINEL: jlong = -1;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_lrurust_LruRustPolicy_lruInitCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	capacity: jlong,
	_debug_mode: jlong, // JNI interface requires this; LRU has no logging
) -> jlong {
	if capacity <= 0 {
		return 0;
	}
	let cache: LruCacheTy = LruCache::new(capacity as usize);
	Box::into_raw(Box::new(cache)) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_lrurust_LruRustPolicy_lruGetFromCacheIfPresent<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	if cache_ptr == 0 {
		return MISS_SENTINEL;
	}
	// SAFETY: Caller (Java) guarantees single-threaded access to this cache pointer.
	// Concurrent calls on the same pointer are undefined behavior.
	let cache = unsafe { &mut *(cache_ptr as *mut LruCacheTy) };
	match cache.get(&key) {
		None => MISS_SENTINEL,
		Some(value) => *value,
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_lrurust_LruRustPolicy_lruPutToCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	// SAFETY: Caller (Java) guarantees single-threaded access to this cache pointer.
	let cache = unsafe { &mut *(cache_ptr as *mut LruCacheTy) };
	cache.put(key, value);
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_lrurust_LruRustPolicy_lruDropCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	// SAFETY: Caller (Java) guarantees this pointer is valid and not already freed.
	unsafe { drop(Box::from_raw(cache_ptr as *mut LruCacheTy)) };
}
