# Cross-Cycle Insights

## Recurring wins

- **Using idle capacity is the only lever that has reliably raised hit-rate** (cycles 4–8). The levers that worked: a two-choice stash (neighbour stash bucket), rescuing stash victims into holes in their normal-tier bucket, and earlier, ordering by reuse inside the stash. All of them kept metadata-size flat. Their gains add up when they draw on different capacity (cycle 5).
- **The stash works as a probationary queue that resists scans** (cycles 1–2). Fresh puts go in at the LRU end and demotees go in at the MRU end. These gains stacked and became the base of every later lineage.
- **Lossless, state-neutral gates are the dependable way to cut get latency** (cycles 6–8). Exact per-segment counters (`borrowed` for the `s1` probe, `foreign` for the probing probe) skip scans, leave hit-rate exactly unchanged, and stack with each other.
- **A counter is free only when its inputs are free** (cycles 6–7). Counters fed from information the caller already knows (slot type, `true_target`) cost nothing on put. Counters that have to re-hash show up in the put tail.
- **State-neutral refactors check out exactly.** Reordering lookups, sharing hashes and plumbing a single hash all reproduced the expected cache state. Trimming the stash-hit → promote → demote chain helps p99 but not p50 (cycles 4, 8).
- **Target-only rescue is the better rescue form** (cycle 8). It keeps nearly all of full rescue's hit-rate and keeps the `foreign` gate clean. Probing-side holes contribute almost nothing.

## Recurring failures

- **Admission filters on either promotion edge** (cycles 2–3). Hotter promotion (stash→target at MRU), a second-hit gate and a native-ownership gate all lost hit-rate. The tiny buckets can't hold items long enough for extra screening to pay off. The current admission policy looks like a local optimum.
- **Fixed lookup-order changes as a latency lever.** The best order flips with placement policy (target-first won in cycle 1, stash-first in cycle 3). With `s1` already probed last, reordering has no headroom left. The remaining cost comes from scanning fuller buckets, not from order (cycle 6).
- **Removing hashes and devirtualising at the bucket level to move get p50** (cycles 4, 7, 8). The median no longer responds to these. A concrete bucket type that still points to a heap `Vec<Item>` made the tails worse.
- **Filling holes eagerly** (cycle 5, fresh puts into target holes; cycle 6, native demotion into probing holes). Eager fills either crowd out promotions that have proven reuse or rarely fire in steady state. Either way they add hot-path branches for no gain.
- **An independently hashed second stash choice** (cycle 7). Stash overflow is barely clustered, so the extra full hash costs more than the small hit-rate it adds.
- **Trusting per-op-time moves that have no code-level cause.** Benchmark drift distorted cycle 1. The parent's put-p99 dip in cycle 7 did not reproduce in cycle 8.

## Open questions

- Would storing slots inline (a fixed `[Item; N]` per bucket, or a flat per-segment array) with a static eviction policy finally make each probe cheaper and move get p50?
- If probes get cheaper, does target-only rescue's latency cost shrink enough to make it a net win? Its cost scales with the number of residents scanned.
- Can rescue be throttled to limit promotion cascades rather than occupancy? Options include rescuing only victims that have shown reuse, or placing rescued items so they yield first.
- Can `borrowed` be maintained entirely from the caller-known `s0` or native flag, so it never re-hashes?
- Would in-bucket fingerprints or tags pay for their metadata-size cost by shortening scans of fuller buckets?
- Stash size and associativity haven't been tested. Bigger stash buckets might let filters that failed on 2-slot buckets work.
- The put-p99 baseline has to be re-measured before any write-path claim can be judged.

## Per-policy notes

- **Stash (probationary tier):** insert fresh items at LRU and demotees at MRU. Two-choice placement via the `s0+1` stride is enough. Probe `s1` last and gate it with `borrowed`. Further stash-side capacity tricks show diminishing returns.
- **Target buckets:** Front placement on stash→target promotion acts as a useful admission filter. Target holes are the main source of rescue's hit-rate. They are also its main source of cost: fuller buckets mean longer scans and more cascades.
- **Probing buckets:** the `foreign` gate makes probing scans skippable on misses. Keep rescue and demotion out of probing holes. Those holes are rare and add little, and writing into them dirties the gate.
- **Trade-off across the whole cache:** at the current per-probe cost, extra residency trades hit-rate directly for get latency. Only cheaper probes (from data layout) can loosen that trade. Metadata-size has not moved in any cycle so far.
