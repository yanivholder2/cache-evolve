//! JNI surface for the Caffeine `AssociativeCacheRustPolicy`.
//!
//! Holds a `Box<dyn framework::Cache<i64, i64>>` constructed via the
//! subject-owned `factory::make_cache`. Variants overlay `src/factory.rs` to
//! substitute their cache; the JNI symbols stay the same.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::factory::make_cache;
use crate::shared::logging;

use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::{info, trace};

type CacheTy = Box<dyn framework::Cache<i64, i64>>;

const NULL_CACHE_PTR: jlong = 0;
const MISS_SENTINEL: jlong = -1;

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
		return NULL_CACHE_PTR;
	};

	logging::init_file_logger("associative", debug_mode as usize);
	let cache: CacheTy = make_cache(&settings);
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
	if cache_ptr == NULL_CACHE_PTR {
		return MISS_SENTINEL;
	}
	// SAFETY: Java caller guarantees single-threaded access per cache pointer.
	let cache = unsafe { &mut *(cache_ptr as *mut CacheTy) };
	match cache.get(&key) {
		None => {
			trace!("associativeGetFromCacheIfPresent: ptr={cache_ptr} key={key} miss");
			MISS_SENTINEL
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
	if cache_ptr == NULL_CACHE_PTR {
		return;
	}
	// SAFETY: Java caller guarantees single-threaded access per cache pointer.
	let cache = unsafe { &mut *(cache_ptr as *mut CacheTy) };
	cache.put(key, value);
	trace!("associativePutToCache: ptr={cache_ptr} key={key} value={value}");
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associative_AssociativeCacheRustPolicy_associativeDropCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
) {
	if cache_ptr == NULL_CACHE_PTR {
		return;
	}
	info!("associativeDropCache: ptr={cache_ptr}");
	// SAFETY: Java caller calls dropCache exactly once per pointer.
	unsafe { drop(Box::from_raw(cache_ptr as *mut CacheTy)) };
}
