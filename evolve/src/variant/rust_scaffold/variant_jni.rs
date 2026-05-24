use crate::VariantSegment;
use dash::dash::dash_settings::DashSettings;
use dash::dash::Dash;
use dash::shared::logging;

use jni::{
	objects::{JClass, JString},
	sys::jlong,
	JNIEnv,
};
use log::{error, info, trace};

type CacheTy = Dash<i64, i64, VariantSegment<i64, i64>>;

// Shared SAFETY contract for every `unsafe { &mut *(cache_ptr as *mut CacheTy) }` site below:
// the Java caller stores the pointer returned by create_cache, passes it back on every JNI call,
// guarantees single-threaded access per pointer, and calls dropCache exactly once per pointer.

const NULL_CACHE_PTR: jlong = 0;
const MISS_SENTINEL: jlong = -1;

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashVariantRustPolicy_initDefaultCache(
	_env: JNIEnv,
	_class: JClass,
) -> jlong {
	create_cache(DashSettings::default())
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashVariantRustPolicy_initCache(
	env: JNIEnv,
	_class: JClass,
	num_of_segments: jlong,
	num_of_normal_buckets: jlong,
	num_of_stash_buckets: jlong,
	bucket_size: jlong,
	eviction_policy: jlong,
	debug_mode: jlong,
	log_path: JString,
) -> jlong {
	let log_path: String = env
		.get_string(log_path)
		.map(Into::into)
		.unwrap_or_default();
	let debug = debug_mode.max(0) as usize;
	if log_path.is_empty() {
		logging::init_file_logger("dash_variant", debug);
	} else {
		logging::init_file_logger_at("dash_variant", debug, &log_path);
	}

	let Some(settings) = DashSettings::from_jni_args(
		num_of_segments,
		num_of_normal_buckets,
		num_of_stash_buckets,
		bucket_size,
		eviction_policy,
		debug_mode,
	) else {
		error!("dash_variant_jni.init invalid args: segments={num_of_segments} buckets={num_of_normal_buckets} stash={num_of_stash_buckets} bucket_size={bucket_size} policy={eviction_policy} debug={debug_mode}");
		return NULL_CACHE_PTR;
	};

	create_cache(settings)
}

fn create_cache(settings: DashSettings) -> jlong {
	// Safety net: if a path reaches create_cache without prior init (e.g. initDefaultCache),
	// fall back to the default rust-logs dir. The file logger uses a shared `Once`, so this
	// is a no-op when the JNI entry (initCache) already initialised the logger at the
	// caller-supplied path.
	logging::init_file_logger("dash_variant", settings.debug_mode);

	let Some(cache) = CacheTy::try_new(settings.num_of_segments, || Some(VariantSegment::new(&settings))) else {
		error!(
			"dash_variant_jni.create_cache: initialization failed. segments={}, buckets={}, stash={}, policy={:?}",
			settings.num_of_segments, settings.num_of_normal_buckets, settings.num_of_stash_buckets, settings.eviction_policy
		);
		return NULL_CACHE_PTR;
	};
	let cache_ptr = Box::into_raw(Box::new(cache)) as jlong;

	info!("create_cache: ptr={cache_ptr} settings={settings:?}");
	cache_ptr
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashVariantRustPolicy_getFromCacheIfPresent(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
) -> jlong {
	if cache_ptr == NULL_CACHE_PTR {
		error!("getFromCacheIfPresent: null cache_ptr (init failed); returning miss for key={key}");
		return MISS_SENTINEL;
	}
	let cache = unsafe { &mut *(cache_ptr as *mut CacheTy) };
	match cache.get_and_update_item(&key) {
		None => {
			trace!("getFromCacheIfPresent: ptr={cache_ptr} key={key} miss");
			MISS_SENTINEL
		}
		Some(value) => {
			trace!("getFromCacheIfPresent: ptr={cache_ptr} key={key} hit");
			*value
		}
	}
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashVariantRustPolicy_putToCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
	key: jlong,
	value: jlong,
) {
	if cache_ptr == NULL_CACHE_PTR {
		error!("putToCache: null cache_ptr (init failed); dropping put for key={key} value={value}");
		return;
	}
	let cache = unsafe { &mut *(cache_ptr as *mut CacheTy) };
	cache.put(key, value);
	trace!("putToCache: ptr={cache_ptr} key={key} value={value}");
}

#[no_mangle]
pub extern "system" fn Java_com_github_benmanes_caffeine_cache_simulator_policy_dash_DashVariantRustPolicy_dropCache(
	_env: JNIEnv,
	_class: JClass,
	cache_ptr: jlong,
) {
	if cache_ptr == NULL_CACHE_PTR {
		error!("dropCache: null cache_ptr (init failed); nothing to drop");
		return;
	}
	info!("dropCache: ptr={cache_ptr}");
	// SAFETY: see CacheTy; Java caller calls dropCache exactly once per pointer.
	unsafe { drop(Box::from_raw(cache_ptr as *mut CacheTy)) };
}
