import subprocess
import os
import csv
import datetime
import argparse
import shutil


def _repo_root():
    return subprocess.check_output(
        ["git", "rev-parse", "--show-toplevel"], text=True
    ).strip()


REPO_ROOT = _repo_root()
CAFFEINE_PATH = os.path.normpath(os.path.join(REPO_ROOT, "..", "caffeine"))
GRADLEW = os.path.join(CAFFEINE_PATH, "gradlew.bat" if os.name == "nt" else "gradlew")
REPORTS_PATH = os.path.join(CAFFEINE_PATH, "simulator", "build", "reports", "simulate")

DEFAULT_TRACES = [
	"lirs:multi1.trace.gz",
	"lirs:multi2.trace.gz",
]

DEFAULT_SIZES = [512, 1024, 1536, 2048, 4096, 8192, 16384, 32768, 65536]
DEFAULT_POLICIES = ["classiclru", "lfu", "fifo"]


def parse_args():
	p = argparse.ArgumentParser(description="Run Caffeine cache simulations across traces and sizes")
	p.add_argument("--traces", type=str, default=None,
		help="Comma-separated traces (e.g., lirs:multi1.trace.gz,lirs:multi2.trace.gz)")
	p.add_argument("--sizes", type=str, default=None,
		help="Comma-separated cache sizes (e.g., 512,1024,4096)")
	p.add_argument("--policies", type=str, default=None,
		help="Comma-separated eviction policies (e.g., lru,lfu,fifo)")
	p.add_argument("--normal-buckets", type=int, default=28)
	p.add_argument("--stash-buckets", type=int, default=4)
	p.add_argument("--bucket-size", type=int, default=12)
	p.add_argument("--debug", action="store_true")
	p.add_argument("--output-dir", type=str, default=None)
	p.add_argument("--skip-build", action="store_true")
	p.add_argument("--dry-run", action="store_true", help="Print commands without executing")
	p.add_argument("--rust-path", type=str, default=None,
		help="Path to target/release dir (auto-detected from git rev-parse if omitted)")
	return p.parse_args()


def resolve_rust_path(args_rust_path):
	"""Resolve the target/release path, handling worktrees."""
	if args_rust_path:
		return args_rust_path
	return os.path.join(REPO_ROOT, "target", "release")


def build_dll():
	print("Building dash.dll...")
	subprocess.run(["cargo", "build", "--release"], check=True)
	print("Build complete.")


def format_sizes(size_list):
	"""Format sizes with underscore separators for Gradle (e.g., 16384 -> 16_384)."""
	return ",".join(f"{s:,}".replace(",", "_") for s in size_list)


def trace_display_name(trace):
	"""'lirs:multi1.trace.gz' -> 'multi1_lirs'."""
	fmt, filename = trace.split(":", 1)
	basename = os.path.basename(filename)
	name = basename.split(".")[0]
	return f"{name}_{fmt}"


def run_simulation(trace, sizes_str, policies, args, rust_path):
	"""Run simulator:simulate for a single trace with all sizes."""
	trace_name = trace_display_name(trace)
	debug_mode = 2 if args.debug else 0
	jvm_args = f"-XX:+UseParallelGC,-Xmx8g,-Djava.library.path={rust_path}"

	cmd = [
		GRADLEW, "-p", CAFFEINE_PATH,
		"simulator:simulate", "-q",
		f"--maximumSize={sizes_str}",
		f"--title={trace_name}",
		f"-PjvmArgs={jvm_args}",
		"-Dcaffeine.simulator.trace.source=files",
		f"-Dcaffeine.simulator.files.paths.0={trace}",
		f"-Dcaffeine.simulator.dash.numOfNormalBuckets={args.normal_buckets}",
		f"-Dcaffeine.simulator.dash.numOfStashBuckets={args.stash_buckets}",
		f"-Dcaffeine.simulator.dash.bucketSize={args.bucket_size}",
		f"-Dcaffeine.simulator.dash.debugMode={debug_mode}",
	]
	for i, policy in enumerate(policies):
		cmd.append(f"-Dcaffeine.simulator.dash.policy.{i}={policy}")

	print(f"\nRunning: {trace_name}")
	if args.debug or args.dry_run:
		print(f"  Command: {' '.join(cmd)}")
	if not args.dry_run:
		subprocess.run(cmd, check=True)
		print(f"  Done: {trace_name}")
	return trace_name


def collect_results(trace_name, output_dir):
	"""Copy simulate output to a per-trace directory before next run overwrites it."""
	trace_dir = os.path.join(output_dir, trace_name)
	os.makedirs(trace_dir, exist_ok=True)

	collected = {}
	for f in os.listdir(REPORTS_PATH):
		if f.startswith("hit_rate") and (f.endswith(".csv") or f.endswith(".png")):
			dst = os.path.join(trace_dir, f)
			shutil.copy2(os.path.join(REPORTS_PATH, f), dst)
			if f == "hit_rate.csv":
				collected["csv"] = dst
			elif f == "hit_rate.png":
				collected["png"] = dst

	return collected


def create_summary_csv(output_dir, trace_results):
	"""Aggregate per-trace combined CSVs into a single summary."""
	summary_path = os.path.join(output_dir, "summary.csv")
	all_rows = []
	header = None

	for trace_name, files in trace_results.items():
		csv_path = files.get("csv")
		if not csv_path or not os.path.exists(csv_path):
			continue
		with open(csv_path, "r", newline="") as f:
			reader = csv.reader(f)
			header = next(reader)
			for row in reader:
				all_rows.append([trace_name] + row)

	if not all_rows or header is None:
		print("No results to summarize.")
		return None

	with open(summary_path, "w", newline="") as f:
		writer = csv.writer(f)
		writer.writerow(["Trace"] + header)
		writer.writerows(all_rows)

	return summary_path


def main():
	args = parse_args()
	rust_path = resolve_rust_path(args.rust_path)

	traces = args.traces.split(",") if args.traces else DEFAULT_TRACES
	sizes = [int(s) for s in args.sizes.split(",")] if args.sizes else DEFAULT_SIZES
	policies = args.policies.split(",") if args.policies else DEFAULT_POLICIES

	sizes_str = format_sizes(sizes)
	timestamp = datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")

	default_output = os.path.join(CAFFEINE_PATH, "simulator", "build", "reports", "sweep")
	output_dir = os.path.join(args.output_dir or default_output, timestamp)
	os.makedirs(output_dir, exist_ok=True)

	if not args.skip_build and not args.dry_run:
		build_dll()

	print(f"\nSweep: {len(traces)} traces x {len(sizes)} sizes")
	print(f"Policies: {policies}")
	print(f"Library: {rust_path}")
	print(f"Output: {output_dir}\n")

	trace_results = {}
	for trace in traces:
		trace_name = run_simulation(trace, sizes_str, policies, args, rust_path)
		if not args.dry_run:
			trace_results[trace_name] = collect_results(trace_name, output_dir)

	if not args.dry_run:
		summary = create_summary_csv(output_dir, trace_results)
		if summary:
			print(f"\nSummary CSV: {summary}")
		print(f"Results: {output_dir}")


if __name__ == "__main__":
	main()
