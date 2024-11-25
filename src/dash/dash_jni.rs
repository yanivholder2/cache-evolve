use crate::settings::EvictionPolicy;

use super::dash_settings::DashSettings;
use super::Dash;

use chrono::Local;
use jni::{objects::JClass, sys::jlong, JNIEnv};
use log::info;
use simplelog::*;
use std::fs::{create_dir_all, File};
use std::sync::Once;

type DashTy = Dash<i64, i64>;

static INIT: Once = Once::new();

fn init_logger(debug_mode: usize) {
	INIT.call_once(|| {
		// Get the current time and format it
		let current_time = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
		let log_folder_name = "rust-logs";
		let log_file_path = format!("{}\\dash_rust_{}.log", log_folder_name, current_time);

		// Create the logs directory if it doesn't exist
		create_dir_all(log_folder_name).unwrap();

		// Determine the log level based on debug_mode
		let log_level = match debug_mode {
			2 | 3 => LevelFilter::Debug,
			1 => LevelFilter::Info,
			0 => LevelFilter::Warn,
			_ => LevelFilter::Info,
		};

		// Initialize the logger
		CombinedLogger::init(vec![WriteLogger::new(
			log_level,
			Config::default(),
			File::create(log_file_path).unwrap(),
		)])
		.unwrap();
	});
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_initDefaultCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
) -> jlong {
	create_cache(DashSettings::default())
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_initCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	num_of_segments: jlong,
	num_of_normal_buckets: jlong,
	num_of_stash_buckets: jlong,
	bucket_size: jlong,
	eviction_policy: jlong,
	debug_mode: jlong,
) -> jlong {
	let settings = DashSettings {
		num_of_segments: num_of_segments as usize,
		num_of_normal_buckets: num_of_normal_buckets as usize,
		num_of_stash_buckets: num_of_stash_buckets as usize,
		bucket_size: bucket_size as usize,
		eviction_policy: EvictionPolicy::from_usize(eviction_policy as usize).unwrap(),
		debug_mode: debug_mode as usize,
	};

	create_cache(settings)
}

fn create_cache(settings: DashSettings) -> jlong {
	init_logger(settings.debug_mode);

	let cache: DashTy = Dash::new(settings.clone());
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;

	info!(
		"new - Cache Ptr: {}, initializing cache. Settings: {:?}",
		cache_ptr, settings
	);
	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_getFromCacheIfPresent<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	let cache = unsafe { &mut *(cache_ptr as *mut DashTy) };
	let res = cache.get_and_update_item(&key);
	match res {
		None => {
			info!("get_and_update_item - Cache Ptr: {}, Key: {}, miss", cache_ptr, key);
			-1
		}
		Some(value) => {
			info!("get_and_update_item - Cache Ptr: {}, Key: {}, hit", cache_ptr, key);
			*value
		}
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_putToCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	let cache = unsafe { &mut *(cache_ptr as *mut DashTy) };
	cache.put(key as i64, value as i64);
	info!("put - Cache Ptr: {}, Key: {}, Value: {}", cache_ptr, key, value);
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashRustPolicy_dropCache<
	'local,
>(
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	cache_ptr: jlong,
) {
	let _boxed_cache = unsafe { Box::from_raw(cache_ptr as *mut DashTy) };
	info!("drop_cache - Cache Ptr: {}", cache_ptr);
}
