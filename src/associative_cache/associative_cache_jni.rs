use super::associative_cache_settings::DEFAULT_SETTINGS;
use super::AssociativeCache;
use crate::shared::traits::cache::Cache;
use jni::{objects::JClass, sys::jlong, JNIEnv};
use once_cell::sync::OnceCell;

type DefaultAssociativeCache = AssociativeCache<i64, i64>;

static mut CACHE: OnceCell<DefaultAssociativeCache> = OnceCell::new();

fn shared_cache() -> &'static mut DefaultAssociativeCache {
	unsafe { CACHE.get_mut().expect("The cache is not initialized") }
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associativeCacheRust_AssociativeCacheRust_initCache(
	_env: JNIEnv,
	_class: JClass,
) {
	unsafe {
		CACHE.set(AssociativeCache::new(DEFAULT_SETTINGS)).expect("");
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associativeCacheRust_AssociativeCacheRust_getFromCacheIfPresent(
	_env: JNIEnv,
	_class: JClass,
	key: jlong,
) -> jlong {
	let res = shared_cache().get(&key);
	match res {
		None => -1,
		Some(value) => value,
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associativeCacheRust_AssociativeCacheRust_putToCache(
	_env: JNIEnv,
	_class: JClass,
	key: jlong,
	value: jlong,
) {
	shared_cache().put(key, value);
}
