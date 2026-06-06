pub mod logging;
pub mod s3_fifo;

pub fn init_file_and_stdout_logger(cache_type: &str, debug_mode: usize) {
	logging::init_file_and_stdout_logger(cache_type, debug_mode);
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
