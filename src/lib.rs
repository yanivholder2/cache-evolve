pub mod associative_cache;
pub mod dash;
pub mod eviction_policy;
pub mod lru_cache;
mod shared;
pub use shared::utils::get_index;

pub fn init_file_and_stdout_logger(cache_type: &str, debug_mode: usize) {
	shared::logging::init_file_and_stdout_logger(cache_type, debug_mode);
}

#[cfg(test)]
mod test_utils;
