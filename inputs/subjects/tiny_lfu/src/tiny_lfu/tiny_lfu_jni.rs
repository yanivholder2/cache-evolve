use super::TinyLfuCache;
use crate::logging;
use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::{info, trace};

type TinyLfuCacheTy = TinyLfuCache<i64, i64>;

const MISS_SENTINEL: jlong = -1;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_tinylfu_TinyLfuRustPolicy_tinyLfuInitCache(
	_env: JNIEnv,
	_class: JClass,
	capacity: jlong,
	debug_mode: jlong,
) -> jlong {
	logging::init_file_logger("tiny_lfu", debug_mode as usize);

	let Some(cache) = TinyLfuCacheTy::try_new(capacity as usize) else {
		return 0;
	};
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;
	info!("tinyLfuInitCache: ptr={cache_ptr} capacity={capacity}");
	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_tinylfu_TinyLfuRustPolicy_tinyLfuGetFromCacheIfPresent(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	if cache_ptr == 0 {
		return MISS_SENTINEL;
	}
	let cache = unsafe { &mut *(cache_ptr as *mut TinyLfuCacheTy) };
	match cache.get(&key) {
		None => {
			trace!("tinyLfuGetFromCacheIfPresent: ptr={cache_ptr} key={key} miss");
			MISS_SENTINEL
		}
		Some(value) => {
			trace!("tinyLfuGetFromCacheIfPresent: ptr={cache_ptr} key={key} hit");
			*value
		}
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_tinylfu_TinyLfuRustPolicy_tinyLfuPutToCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	let cache = unsafe { &mut *(cache_ptr as *mut TinyLfuCacheTy) };
	cache.put(key, value);
	trace!("tinyLfuPutToCache: ptr={cache_ptr} key={key} value={value}");
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_tinylfu_TinyLfuRustPolicy_tinyLfuDropCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	info!("tinyLfuDropCache: ptr={cache_ptr}");
	unsafe { drop(Box::from_raw(cache_ptr as *mut TinyLfuCacheTy)) };
}
