## Recurring wins

- **Removing whole units of work beats re-implementing them.** The wins that held up all deleted work outright instead of making it cheaper. Examples: the byte-tag pre-filter cut full-key compares (0005), the fused rotation removed a memmove (0006), a single multiplicative hash replaced SipHash (0009), and devirtualising promote dropped the virtual call (0010). Each one helped on both p50 and p99.
- **Dense data on the hot path helps; scattered data hurts.** A byte-tag stream works where a `u64` key mirror failed. Narrowing `Item` fields (24 B in 0004, 16 B in 0010) is the only lever that has moved metadata-size.
- **Fast paths that keep semantics exact win cleanly.** An exact fast path such as "already at MRU, return" gives the speed gain with bit-identical hit rate. Skipping work in a way that changes semantics costs some hit rate plus a branch.
- **The replacement policy is now stable.** Scan resistance (quarter-depth LIP plus BIP admission, or a CLOCK reference bit) was the only big hit-rate lever, and it was taken in 0001. Since then, every mechanical rewrite has matched all hit-rate cells exactly, so speed work on a frozen policy is low-risk for hit rate.
- **Cost often sits outside the code being tuned.** The largest hidden costs were a per-op SipHash above the bucket and dispatch plus tag fix-up on put-update. Both were found only after several cycles of tuning in-bucket loops. Look at the whole op path before micro-tuning.

## Recurring failures

- **Reuse and adaptation signals (five attempts).** A fixed ghost ring, a hashed ghost bit set, adaptive depth from hit position, an unbounded second-chance bit, and lazy promotion that changes order all failed the hm_0 vs web_0 tradeoff, or saturated on wide buckets. On wide buckets the BIP period is a zero-sum knob, and 1/32 → 1/64 → 1/128 is now closed.
- **"Clever" rewrites of short loops.** `rotate_left`, variable-length `ptr::copy` on the hit path, and SWAR tag scans all lost to simple loops the compiler already optimised well. Hand-written moves only pay off when they remove an entire memmove.
- **Adding state or writes to skip cheap work.** Miss memos, a dense key Vec, ghost rings, and stores on the get path all added cost. The extra state dirties cache lines, grows the stride, or splits memory streams, and that outweighs the work it skips.
- **Scan order and hit-time reordering.** Reversed scans and self-organising swaps did not move get p50.
- **Expecting a speed win from removing the atomic clock.** On the tag/fused base the atomic was uncontended and already cheap. Removing it is justified for correctness only.
- **Trusting single-run latency.** Unchanged paths have drifted in most cycles since 0004, and get-p99 is especially noisy. A conclusion based on one run has been reversed later more than once.
- **Reading flat metadata-size as "free".** The metric only sees `Item` layout. It misses per-bucket side buffers, ghosts, and extra fields.

## Open questions

- Do 16-byte `Item`s speed up the put-miss rotation and insert memmoves once they sit on the devirtualised base? Was 0010/v003's regression caused by the widened dynamic signature, or by drift?
- What dominates get now: the full-key compare, bucket indexing (`%` vs mask), or something else? Profile it before writing another hypothesis.
- What is left on put-miss after the fused rotation: the quarter-length shift, the tag mirror, or the insert? Could a ring offset for the probationary region avoid the shift?
- Does bucket stride cost anything on its own? This needs a padding-only control.
- Should every per-op claim go through a repeated same-session A/B against the parent? This hasn't been made routine yet.

## Per-policy notes

- **ClassicLRU / LIP+BIP (main line):** Quarter-depth insertion with BIP admission is the best policy, using width-keyed periods of 1/32 for narrow buckets and 1/64 for wide ones. The policy is frozen. The current base is single multiplicative hash + byte tags + fused rotation + clock-free + devirtualised `lip_promote`. On this base, in-bucket get promotion is close to the floor.
- **CLOCK / GCLOCK:** The single-store hit path is its strength. Any reordering on hit throws that advantage away. Extra GCLOCK levels gave a small but consistent hit-rate gain. Bounding the sweep (a 16-slot cap) helped the put tail. It hasn't been explored since 0002, and the byte tags have not been ported to it.
- **Frequency-aware victim selection (0001/v003):** Its result was bit-identical to the baseline, probably because the harness never ran the overlaid eviction path. It has never been properly tested, and harness coverage needs confirming before any retry.
- **Reuse-marker variants (ghost, second chance):** These should only be revisited with an explicit per-item probationary bit that has independent, symmetric decay and spares items outside the admission region.
