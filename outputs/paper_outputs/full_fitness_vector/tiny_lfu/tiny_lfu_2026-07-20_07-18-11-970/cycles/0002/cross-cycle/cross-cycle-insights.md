## Recurring wins

- **Idiomatic Rust abstractions outperform hand-rolled alternatives.** Across both cycles, the compiler's codegen for standard constructs (idiomatic `Option<NonZeroU32>` match arms, `Vec<Vec<u32>>` indirection) consistently beat explicit sentinel arithmetic and serial hash chains. Structural micro-optimizations are routinely absorbed or inverted by the compiler.
- **The baseline's sketch and node design is well-calibrated.** Four rows, separate per-row heap segments, and `Option<NonZeroU32>` node encoding have each survived challenge and either matched or bested every proposed simplification. These choices should be treated as settled.

## Recurring failures

- **Sketch structural simplifications consistently fail.** Flattening to a single allocation raised cache-set pressure enough to more than double latency. Halving row depth degraded collision suppression and cascaded into eviction errors. Neither direction produces wins; the four-row multi-allocation baseline is not over-engineered for this trace.
- **Aggressive sketch decay destroys the frequency signal on a stable working set.** The LIRS trace's long-lived hot set depends on accumulated frequency counts. A `width/2` reset cadence erases that signal faster than accesses can rebuild it, inverting the expected admission-quality gain.
- **Node layout is now an exhausted axis.** `NonZeroU32` compaction recovered the prior cycle's latency regression but landed at baseline parity, not a win. Two cycles of node-encoding work produced no fitness improvement; further layout exploration is unlikely to break through.
- **Serial hash chains lose to ILP-friendly alternatives.** The `h1→h2` dependency chain trades ILP for fewer hash calls; at this key type and scale the data dependency dominates, making the net result a latency regression.

## Open questions

- **Adaptive or epoch-gated sketch decay.** The `width/2` cadence is too aggressive; no test has yet tried a much longer fixed window or an entropy/frequency-variance trigger. This is the single highest-priority untested thread from Cycle 0001.
- **Single-pass seeded hashing.** Deriving all four sketch row positions from one seeded hash (multiply-shift or similar) would eliminate the serial dependency that killed the Cycle 0001 hash-reduction variant. The correctness/collision trade-off at this sketch width is unknown.
- **Admission and eviction policy parameters.** Window-size fraction and protected/probationary segment ratio are entirely unexplored. These are orthogonal to the sketch and node axes now both closed, and may offer the next available fitness headroom.

## Per-policy notes

- **TinyLFU / W-TinyLFU (the only policy tested so far).** The LIRS trace's Zipfian, stable-working-set access pattern makes frequency accumulation a reliable admission signal and penalizes anything that resets or distorts counts prematurely. The CountMinSketch depth (four rows) is load-bearing collision suppression, not redundancy — removing rows degrades hit rate measurably at this sketch width. Separate heap segments per row happen to avoid the cache-set conflicts that a flat layout incurs, making the multi-allocation design a practical necessity rather than a stylistic choice.
