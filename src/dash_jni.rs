use jni::{
    objects::JClass,
    sys::{jlong},
    JNIEnv,
};

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_initCache(
    _env: JNIEnv,
    _class: JClass,
) {

}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_getFromCacheIfPresent(
    _env: JNIEnv,
    _class: JClass,
    key: jlong,
) -> jlong {
    -1
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_putToCache(
    _env: JNIEnv,
    _class: JClass,
    key: jlong,
    value: jlong,
) {

}