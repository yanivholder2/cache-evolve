use chrono::Local;
use log::LevelFilter;
use simplelog::*;
use std::fs::{create_dir_all, File};
use std::sync::Once;

static INIT: Once = Once::new();

fn debug_mode_to_level(debug_mode: usize) -> LevelFilter {
	match debug_mode {
		0 => LevelFilter::Warn,
		1 => LevelFilter::Info,
		2 | 3 => LevelFilter::Debug,
		4 => LevelFilter::Trace,
		_ => LevelFilter::Info,
	}
}

fn build_config() -> Config {
	ConfigBuilder::new()
		.set_target_level(LevelFilter::Error)
		.set_location_level(LevelFilter::Debug)
		.set_thread_level(LevelFilter::Off)
		.set_time_format_str("%Y-%m-%d %H:%M:%S")
		.set_time_to_local(true)
		.set_level_padding(LevelPadding::Left)
		.build()
}

const DEFAULT_LOG_DIR: &str = "rust-logs";

fn create_log_file(cache_type: &str, log_dir: &str) -> Option<File> {
	if let Err(e) = create_dir_all(log_dir) {
		eprintln!("WARN: failed to create {log_dir} dir: {e}");
		return None;
	}
	let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
	let path = format!("{log_dir}/{cache_type}_{timestamp}.log");
	match File::create(&path) {
		Ok(f) => Some(f),
		Err(e) => {
			eprintln!("WARN: failed to create log file {path}: {e}");
			None
		}
	}
}

pub fn init_file_logger(cache_type: &str, debug_mode: usize) {
	INIT.call_once(|| {
		let Some(log_file) = create_log_file(cache_type, DEFAULT_LOG_DIR) else {
			return;
		};
		let level = debug_mode_to_level(debug_mode);
		let config = build_config();
		if let Err(e) = CombinedLogger::init(vec![WriteLogger::new(level, config, log_file)]) {
			eprintln!("WARN: failed to init logger: {e}");
		}
	});
}

pub fn init_file_and_stdout_logger(cache_type: &str, debug_mode: usize) {
	INIT.call_once(|| {
		let level = debug_mode_to_level(debug_mode);
		let config = build_config();
		let mut loggers: Vec<Box<dyn SharedLogger>> = vec![SimpleLogger::new(level, config.clone())];
		if let Some(log_file) = create_log_file(cache_type, DEFAULT_LOG_DIR) {
			loggers.push(WriteLogger::new(level, config, log_file));
		}
		if let Err(e) = CombinedLogger::init(loggers) {
			eprintln!("WARN: failed to init logger: {e}");
		}
	});
}
