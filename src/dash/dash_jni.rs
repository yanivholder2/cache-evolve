use crate::shared::settings::EvictionPolicy;

use super::dash_settings::DashSettings;
use super::dash_settings::DEFAULT_SETTINGS;
use super::Dash;

use chrono::Local;
use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::info;
use once_cell::sync::OnceCell;
use simplelog::*;
use std::fs::{create_dir_all, File};

static mut CACHE: OnceCell<Dash<i64, i64>> = OnceCell::new();

fn shared_cache() -> &'static mut Dash<i64, i64> {
	unsafe { CACHE.get_mut().expect("The cache is not initialized") }
}

fn init_logger() {
	// Get the current time and format it
	let current_time = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
	let log_folder_name = "rust-logs";
	let log_file_path = format!("{}\\dash_rust_{}.log", log_folder_name, current_time);

	// Create the logs directory if it doesn't exist
	create_dir_all(log_folder_name).unwrap();

	// Initialize the logger
	CombinedLogger::init(vec![WriteLogger::new(
		LevelFilter::Info,
		Config::default(),
		File::create(log_file_path).unwrap(),
	)])
	.unwrap();
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_initDefaultCache(
	_env: JNIEnv,
	_class: JClass,
) {
	init_logger();

	info!("Initializing default cache. Settings: {:?}", DEFAULT_SETTINGS);

	unsafe {
		CACHE.set(Dash::new(DEFAULT_SETTINGS)).expect("");
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_initCache(
	_env: JNIEnv,
	_class: JClass,
	num_of_segments: jlong,
	num_of_normal_buckets: jlong,
	num_of_stash_buckets: jlong,
	bucket_size: jlong,
	eviction_policy: jlong,
) {
	init_logger();
	let settings = DashSettings {
		num_of_segments: num_of_segments as usize,
		num_of_normal_buckets: num_of_normal_buckets as usize,
		num_of_stash_buckets: num_of_stash_buckets as usize,
		bucket_size: bucket_size as usize,
		eviction_policy: EvictionPolicy::from_usize(eviction_policy as usize).unwrap(),
	};

	info!("Initializing cache. Settings: {:?}", settings);

	unsafe {
		CACHE.set(Dash::new(settings)).expect("");
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_getFromCacheIfPresent(
	_env: JNIEnv,
	_class: JClass,
	key: jlong,
) -> jlong {
	let res = shared_cache().get_and_update_item(&key);
	match res {
		None => -1,
		Some(value) => *value,
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_putToCache(
	_env: JNIEnv,
	_class: JClass,
	key: jlong,
	value: jlong,
) {
	shared_cache().put(key as i64, value as i64);
}
