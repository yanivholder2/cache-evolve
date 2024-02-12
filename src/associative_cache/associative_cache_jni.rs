/*
use jni::{objects::JClass, sys::jlong, JNIEnv};
use once_cell::sync::OnceCell;
static mut CACHE: OnceCell<MyAssociativeCache> = OnceCell::new();

fn shared_cache() -> &'static mut MyAssociativeCache {
	unsafe { CACHE.get_mut().expect("The cache is not initialized") }
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associativeCacheRust_AssociativeCacheRust_initCache(
	_env: JNIEnv,
	_class: JClass,
) {
	unsafe {
		CACHE.set(MyAssociativeCache::default()).expect("");
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
		Some(value) => *value,
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_associativeCacheRust_AssociativeCacheRust_putToCache(
	_env: JNIEnv,
	_class: JClass,
	key: jlong,
	value: jlong,
) {
	shared_cache().insert(key, value);
}
 */
