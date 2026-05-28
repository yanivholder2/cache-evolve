//! Variant-overridable cache constructor.
//!
//! The baseline returns the default Dash topology with `PromotionColdSegment`.
//! Variants overlay this file to substitute their own `Segment` (or a different
//! cache layout entirely) while keeping the JNI surface untouched.

use crate::dash::dash_settings::DashSettings;
use crate::dash::segments::PromotionColdSegment;
use crate::dash::Dash;

pub fn make_cache(settings: &DashSettings) -> Box<dyn framework::Cache<i64, i64>> {
	let cache = Dash::<i64, i64, PromotionColdSegment<i64, i64>>::new(
		settings.num_of_segments,
		|| PromotionColdSegment::new(settings),
	);
	Box::new(cache)
}
