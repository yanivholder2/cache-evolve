//! Variant-overridable cache constructor.
//!
//! The baseline returns the default `AssociativeCache` topology. Variants overlay
//! this file to substitute their own bucket layout (or a different cache entirely)
//! while keeping the JNI surface untouched.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::associative_cache::AssociativeCache;

pub fn make_cache(settings: &AssociativeCacheSettings) -> Box<dyn framework::Cache<i64, i64>> {
	let cache = AssociativeCache::<i64, i64>::new(*settings);
	Box::new(cache)
}
