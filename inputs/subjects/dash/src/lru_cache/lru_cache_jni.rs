use super::LruCache;
use crate::shared::logging;

use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::{info, trace};

type LruCacheTy = LruCache<i64, i64>;

const MISS_SENTINEL: jlong = -1;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_lrurust_LruRustPolicy_lruInitCache(
	_env: JNIEnv,
	_class: JClass,
	capacity: jlong,
	debug_mode: jlong,
) -> jlong {
	logging::init_file_logger("lru", debug_mode as usize);

	let Some(cache) = LruCacheTy::try_new(capacity as usize) else {
		return 0;
	};
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;
	info!("lruInitCache: ptr={cache_ptr} capacity={capacity}");
	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_lrurust_LruRustPolicy_lruGetFromCacheIfPresent(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	if cache_ptr == 0 {
		return MISS_SENTINEL;
	}
	// SAFETY: Java caller guarantees single-threaded access per cache pointer.
	let cache = unsafe { &mut *(cache_ptr as *mut LruCacheTy) };
	match cache.get(&key) {
		None => {
			trace!("lruGetFromCacheIfPresent: ptr={cache_ptr} key={key} miss");
			MISS_SENTINEL
		}
		Some(value) => {
			trace!("lruGetFromCacheIfPresent: ptr={cache_ptr} key={key} hit");
			*value
		}
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_lrurust_LruRustPolicy_lruPutToCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	// SAFETY: Java caller guarantees single-threaded access per cache pointer.
	let cache = unsafe { &mut *(cache_ptr as *mut LruCacheTy) };
	cache.put(key, value);
	trace!("lruPutToCache: ptr={cache_ptr} key={key} value={value}");
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_lrurust_LruRustPolicy_lruDropCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	info!("lruDropCache: ptr={cache_ptr}");
	// SAFETY: Java caller guarantees single-threaded access and calls dropCache exactly once.
	unsafe { drop(Box::from_raw(cache_ptr as *mut LruCacheTy)) };
}
