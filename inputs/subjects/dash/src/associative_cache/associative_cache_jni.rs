use super::associative_cache_settings::AssociativeCacheSettings;
use super::AssociativeCache;
use crate::shared::logging;

use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::{info, trace};

type AssociativeCacheTy = AssociativeCache<i64, i64>;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_associativeInitCache(
	_env: JNIEnv,
	_class: JClass,
	num_of_buckets: jlong,
	bucket_size: jlong,
	eviction_policy: jlong,
	debug_mode: jlong,
) -> jlong {
	let Some(settings) = AssociativeCacheSettings::from_jni_args(
		num_of_buckets,
		bucket_size,
		eviction_policy,
	) else {
		eprintln!("ERROR: assoc_jni.init invalid args: num_of_buckets={num_of_buckets}, bucket_size={bucket_size}, eviction_policy={eviction_policy}");
		return 0;
	};

	logging::init_file_logger("associative", debug_mode as usize);
	let Some(cache) = AssociativeCacheTy::try_new(settings) else {
		return 0;
	};
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;
	info!("associativeInitCache: ptr={cache_ptr} settings={settings:?}");
	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_associativeGetFromCacheIfPresent(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	if cache_ptr == 0 {
		return -1;
	}
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
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_associativePutToCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	let cache = unsafe { &mut *(cache_ptr as *mut AssociativeCacheTy) };
	cache.put(key, value);
	trace!("associativePutToCache: ptr={cache_ptr} key={key} value={value}");
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_associativeDropCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
) {
	if cache_ptr == 0 {
		return;
	}
	info!("associativeDropCache: ptr={cache_ptr}");
	unsafe { drop(Box::from_raw(cache_ptr as *mut AssociativeCacheTy)) };
}
