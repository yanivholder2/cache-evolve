use crate::s3_fifo::S3Fifo;
use crate::test_utils::BASELINE_CAPACITY;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{TestError, TestRunner};

pub struct S3FifoInvariants;

impl framework::Invariants for S3FifoInvariants {
	fn run_invariant_suite() -> Result<(), String> {
		run_invariant_suite(BASELINE_CAPACITY)
	}
}

pub fn run_invariant_suite(capacity: usize) -> Result<(), String> {
	TestRunner::default()
		.run(&vec(0i64..200, 1..200), |ops| {
			let mut cache: S3Fifo<i64, i64> = S3Fifo::new(capacity);
			for &key in &ops {
				if cache.get(&key).is_none() {
					cache.put(key, key);
				}
			}
			let resident = (0i64..200).filter(|k| cache.get(k).is_some()).count();
			prop_assert!(
				resident <= capacity,
				"capacity_bound violated: {resident} residents but capacity={capacity}"
			);
			Ok(())
		})
		.map_err(|e| format_err("P1 capacity_bound", e))?;

	TestRunner::default()
		.run(&vec(0i64..200, 1..200), |ops| {
			let mut cache: S3Fifo<i64, i64> = S3Fifo::new(capacity);
			for &key in &ops {
				if cache.get(&key).is_none() {
					cache.put(key, key * 10);
				}
			}
			for key in 0i64..200 {
				if let Some(&value) = cache.get(&key) {
					prop_assert!(
						value == key * 10,
						"value_fidelity violated: key={key} read {value}, expected {}",
						key * 10
					);
				}
			}
			Ok(())
		})
		.map_err(|e| format_err("P2 value_fidelity", e))?;

	TestRunner::default()
		.run(&vec(0i64..100, 1..200), |ops| {
			let mut cache: S3Fifo<i64, i64> = S3Fifo::new(capacity);
			for &key in &ops {
				if cache.get(&key).is_none() {
					cache.put(key, key);
				}
			}
			for phantom in [1000i64, 2000, 3000, -1, -50] {
				prop_assert!(
					cache.get(&phantom).is_none(),
					"no_phantom_items violated: never-put key={phantom} returned a value"
				);
			}
			Ok(())
		})
		.map_err(|e| format_err("P3 no_phantom_items", e))?;

	Ok(())
}

fn format_err(invariant: &str, e: TestError<Vec<i64>>) -> String {
	format!("{invariant}: {e}")
}
