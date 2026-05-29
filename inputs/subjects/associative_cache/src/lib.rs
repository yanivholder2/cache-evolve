pub mod associative_cache;
pub mod factory;
pub mod fitness_json;
pub mod shared;
pub use shared::utils::get_index;

pub fn init_file_and_stdout_logger(cache_type: &str, debug_mode: usize) {
	shared::logging::init_file_and_stdout_logger(cache_type, debug_mode);
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
