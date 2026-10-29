#!/usr/bin/env python3
"""Convert an MSR Cambridge block trace (SNIA IOTTA) into lirs format (one key per line).

Caffeine's own `snia-cambridge` reader is WEIGHTED (key = byte offset, weight =
byte size), which none of the Rust subject policies support. This expands every
request into the fixed-size blocks it touches and emits one unweighted key per
block, so the trace runs through the plain `lirs` reader like workload-a.

Input rows: Timestamp,Hostname,DiskNumber,Type,Offset,Size,ResponseTime
(gzip CSV, Offset/Size in bytes). Key = Offset // block_size (+ following blocks).

    .venv/bin/python scripts/convert_msr_trace.py traces/msr/raw/web_0.csv.gz \
        traces/msr/web_0.trace.gz [--block-size 4096] [--reads-only] [--max-events N]
"""

import argparse
import csv
import gzip
from pathlib import Path

BLOCK_SIZE = 4096  # bytes per cache key (page-sized block)


def convert(src: Path, dst: Path, block_size: int, reads_only: bool, max_events: int | None) -> None:
    requests = events = skipped = 0
    keys: set[int] = set()
    dst.parent.mkdir(parents=True, exist_ok=True)
    with gzip.open(src, "rt", newline="") as inp, gzip.open(dst, "wt") as out:
        for row in csv.reader(inp):
            if len(row) != 7:
                raise ValueError(f"{src}: expected 7 fields, got {len(row)}: {row!r}")
            _, _, _, kind, offset_str, size_str, _ = row
            if kind not in ("Read", "Write"):
                raise ValueError(f"{src}: unknown request type {kind!r}")
            if reads_only and kind != "Read":
                continue
            offset, size = int(offset_str), int(size_str)
            if size <= 0:
                skipped += 1
                continue
            requests += 1
            first, last = offset // block_size, (offset + size - 1) // block_size
            for key in range(first, last + 1):
                out.write(f"{key}\n")
                keys.add(key)
                events += 1
                if max_events is not None and events >= max_events:
                    break
            if max_events is not None and events >= max_events:
                break

    print(
        f"wrote {events} events from {requests} requests ({skipped} zero-size skipped) over "
        f"{len(keys)} distinct {block_size}-byte blocks into {dst}"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("src", type=Path, help="MSR <host>_<disk>.csv.gz")
    parser.add_argument("dst", type=Path, help="output lirs trace (.trace.gz)")
    parser.add_argument("--block-size", type=int, default=BLOCK_SIZE, help="bytes per key (default 4096)")
    parser.add_argument("--reads-only", action="store_true", help="drop Write requests")
    parser.add_argument("--max-events", type=int, help="stop after this many output keys")
    args = parser.parse_args()
    convert(args.src, args.dst, args.block_size, args.reads_only, args.max_events)


if __name__ == "__main__":
    main()
