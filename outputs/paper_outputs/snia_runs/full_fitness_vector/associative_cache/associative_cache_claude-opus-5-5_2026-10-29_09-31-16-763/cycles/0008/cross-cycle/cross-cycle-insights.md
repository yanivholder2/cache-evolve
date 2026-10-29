# Cross-Cycle Insights (through cycle 0008)

## Recurring wins

- **Scan-resistant insertion is the only hit-rate lever that paid off big.** LIP quarter-depth insertion and CLOCK clear-on-insert (cycle 0001) took most of the available gain. Every later policy change was a small trade between workloads. The gains are largest on wide buckets.
- **Cutting whole units of work wins; making a loop "cleverer" does not.** The two clear speed wins both removed work outright. The dense byte-tag pre-filter (0005) cut full-key compares on the miss-dominated scan. The fused evict+insert rotation (0006) removed a whole memmove from full-bucket puts.
- **Dense side data works; sparse side data does not.** A byte-tag stream 24× denser than items, scanned in chunks the compiler can vectorise, succeeded. A `u64` key mirror (0002) failed because it added a second memory stream to every hit.
- **Shrinking `Item` is the only thing that moves metadata-size** (0004, 24-byte item). The atomic tick that came with it caused the regression, not the denser layout (0005).
- **Mechanical rewrites keep hit rate exactly.** Rotations, narrower fields, clock removal, tags and hit-path rewrites have matched every hit-rate cell bit-for-bit in five or more cycles. The risk sits entirely in per-op time.
- **Splitting policy by bucket width is zero-cost and safe.** Width-keyed BIP left the narrow cells bit-identical and gave a small net gain on wide buckets (0007–0008).

## Recurring failures

- **Reuse and adaptation signals (five tries, all failed the hm_0/web_0 tradeoff).** Tried: fixed 16-entry ghost ring (0003), hashed ghost bit set (0004, saturated and became a random MRU promoter), adaptive depth from hit position (0005, the position was not a real signal), a second-chance bit with no decay (0006, saturated on large caches), and BIP period tuning (closed in 0008, gains and losses scale together). A signal sized to one workload hurts the other.
- **Reordering on hits, or changing scan order.** Reversing the scan did nothing. Self-organizing swaps turned a one-byte store into writes across two cache lines. Get cost tracks the number of compares, not where the hit sits.
- **Extra work on the put-miss path.** Sweeps, ghost lookups, SipHash, dynamic policy calls on every miss and the shared atomic each added per-op cost. Adding something to bound the tail has always raised p50.
- **"Clever" low-level rewrites.** Each of these was slower than the simple compiler-friendly loop it replaced: `rotate_left(1)` (0007, scalar slow path), variable-length `ptr::copy` on the hit path (0008, likely an out-of-line memmove), and SWAR tag scan (0008, fixed per-chunk overhead plus a branchy mask loop).
- **Removing the atomic clock as a speed lever.** On the tag/fused base it was speed-neutral even with bucket stride pinned (0006–0007). The uncontended atomic stays in L1 and is cheap. Keep the change only for correctness.
- **Trusting single-run latency.** Per-op-time drifted in cycles 0004, 0006, 0007 and 0008. Variants slowed on code paths they did not touch, and get-p99 swung with no code change behind it.

## Open questions

- Where does the remaining miss-path time go? Measure tag-scan length, the key compare, rotation, and bucket hash/indexing before writing the next speed hypothesis.
- Is the parent's hit path already near-optimal? Would a fixed-width or const-generic copy stay inline, as confirmed by an assembly check?
- Can the quarter-length shift left after fused rotation be removed, for example with a ring offset for the probationary region or by moving only tags on the hot path?
- Does bucket stride or layout (extra fields growing the struct) hide a cost? This was suspected in 0006 and never isolated.
- Can tags be extended to the CLOCK path?
- How much of each cycle's latency movement survives repeated same-session runs against the parent?

## Per-policy notes

- **ClassicLRU / LIP (quarter-depth) + BIP:** This is the current base policy. It is frozen at fixed quarter depth with BIP 1/32 on narrow buckets and 1/64 on wide ones. Treat the wide-bucket BIP rate as a zero-sum knob between web_0 and hm_0, and do not tune it further. The base carries the byte tags, fused rotation and the clock-free 24-byte item. Fusing evict and insert helped. Rewriting the hit path (rotate_left, variable `ptr::copy`) hurt.
- **CLOCK / GCLOCK:** The single-store hit path is its main advantage. Any reordering on hits gives that advantage back. A bounded 16-slot sweep with clear-on-insert helps the put tail. Extra GCLOCK levels add small but consistent hit-rate gains. Not yet tested: tags on this path, and a dense reference-bit array for the sweep.
- **Frequency-aware victim selection (0001 v003):** It produced bit-identical results, which suggests the harness never ran it. It was never fairly tested. Confirm that the harness runs the overlaid eviction path before retrying it.
- **Second-chance / reuse-marker variants:** Retry only with explicit, independent, symmetric decay, and with spared items placed outside the admission region.
- **Metadata-size (all policies):** It sees only the `Item` layout. Tags, ghosts, counters and other per-bucket state read flat, so a flat reading does not mean a change is free.
