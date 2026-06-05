use super::S3Fifo;
use crate::logging;
use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::{info, trace};

type S3FifoCache = S3Fifo<i64, i64>;

const MISS_SENTINEL: jlong = -1;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_s3fifo_S3FifoPolicy_initCache(
	_env: JNIEnv,
	_class: JClass,
	capacity: jlong,
	debug_mode: jlong,
) -> jlong {
	logging::init_file_logger("s3_fifo", debug_mode as usize);

	let Some(cache) = S3FifoCache::try_new(capacity as usize) else {
		return 0;
	};
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;
	info!("s3Fifo initCache: ptr={cache_ptr} capacity={capacity}");
	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_s3fifo_S3FifoPolicy_getFromCacheIfPresent(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	if cache_ptr == 0 {
		return MISS_SENTINEL;
	}
	let cache = unsafe { &mut *(cache_ptr as *mut S3FifoCache) };
	match cache.get(&key) {
		None => {
			trace!("s3Fifo getFromCacheIfPresent: ptr={cache_ptr} key={key} miss");
			MISS_SENTINEL
		}
		Some(value) => {
			trace!("s3Fifo getFromCacheIfPresent: ptr={cache_ptr} key={key} hit");
			*value
		}
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_s3fifo_S3FifoPolicy_putToCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	let cache = unsafe { &mut *(cache_ptr as *mut S3FifoCache) };
	cache.put(key, value);
	trace!("s3Fifo putToCache: ptr={cache_ptr} key={key} value={value}");
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_s3fifo_S3FifoPolicy_dropCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	info!("s3Fifo dropCache: ptr={cache_ptr}");
	unsafe { drop(Box::from_raw(cache_ptr as *mut S3FifoCache)) };
}
