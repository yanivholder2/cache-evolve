# Cross-Cycle Insights (through cycle 0008)

## Recurring wins

- **The adaptive small/main split is the durable policy core.** It has held up for eight cycles. Static splits (0003 10/90, 0004 frozen 10/90), ARC-direction rewrites and wholesale policy replacements all lost to it, and it soaks up perturbations to other levers.
- **Hit-rate moves only through the policy's bounds or depth, never through trigger rates.** Two levers have worked: the lower floor clamp (`capacity/64`, cycle 0005, reproduced in 0006) and retention depth (`MAX_FREQ` 3→7, cycle 0008). Both gains were small but real.
- **Policy and layout levers are independent and stack.** The floor's gain reproduced exactly on another layout, and the layout refactors kept policy unchanged. Winning levers can be combined across lineages.
- **For `get` latency, touch only memory the probe already reads.** Three layout changes followed this rule and won: hashed indexing replacing linear scans (0002), moving cold fields such as the key out of the hot array behind a narrow index (0006), and bit-packing `freq` into the index bucket (0008). The current best layout is the narrow `(K, u32)` index with packed `freq`, an 8-byte `Entry`, and keys in a separate array.
- **Most `put` gains that hold up trace to removed machine work.** Examples are dropping clones and `retain` compaction (the fingerprint ring, 0003), replacing modulo with a compare, and inlining eviction (0004). Count the hashes and clones in the parent before claiming them as a lever.

## Recurring failures

- **Making the ghost path stronger backfires.** This has happened three times: ghosting main victims (0002), readmitting with extra `freq` credit (0005), and growing the ghost to full capacity (0008). Each one readmits keys whose reuse distance is too long and pushes out better residents. Shrinking the ghost (0004) was roughly neutral because the adaptation absorbed it.
- **Trigger-rate tuning does nothing on workload-a.** Damping grow-small (0006) and doubling the shrink step (0007) were both inert. Loosening the graduation threshold (0002) was flat as well.
- **Two-lever variants can't be read.** In 0003 (H01 and H03), and in 0004 when ghost size was tied to the split, there was no way to attribute the losses. Pin the ghost size whenever the split is touched.
- **The wrong kind of layout change hurts `get`:**
  - An inline payload in the probe table makes buckets wide and not a power of two, and slowed `get` in 0005 and 0006.
  - Splitting fields that are read together into parallel arrays, like `freq` in 0007, adds a second access on every hit.
- **Micro-trims below the noise floor are wasted cycles.** Store elision on `get` (0007) is the example.
- **Measurement has blind spots:**
  - Credit for `get` latency is unreliable when the `get` path is byte-identical. This happened repeatedly, and drift reached `put` in 0007.
  - Eviction work on the `put` side leaks into `get` in the mixed benchmark.
  - `metadata-size` counts only `Entry`, so it misses ghost heap and parallel arrays.
- **The infrastructure has failed to deliver data:**
  - Builds failed on Maven 429s in 0001.
  - The settled `small_size` figure from the split reports has never reached an insight in eight cycles.

## Open questions

- **Where does `small_size` settle, and does the floor bind (`floor_blocks`)?** The loop has to copy the split-report stdout into eval artifacts the insight writers can read.
- **Can the `MAX_FREQ` hit-rate gain be kept with a cheaper main drain?** Candidates are halving decay instead of decrementing, or a cap of 4–5 packed as 3 bits (a 29-bit slot id makes room).
- **Would a stricter readmission rule help where every stronger one hurt?** For example, readmit only on a second ghost hit, or only while small is at its floor.
- **Does a niche-free `keys: Vec<K>` cut the real per-slot footprint?** Liveness would be tracked through `free`.
- **Are the recent `put` gains (0004 v003, 0008 v001) real?** That needs same-run parent-vs-child benchmarks.
- **Is the upper clamp (`capacity/2`) a live bound,** the way the floor turned out to be?

## Per-policy notes

- **S3-FIFO with adaptive split (incumbent lineage):** two-touch (`freq > 1`) graduation, ghost-hit shrink, a "main full on graduation" grow trigger, and clamps. The balance point appears to be set by the ghost-hit shrink side and the bounds. The current parent is 0008 v001: the `/64` floor, a fingerprint-ring ghost, the narrow index with packed 2-bit `freq`, and an 8-byte `Entry`.
- **Static-split S3-FIFO (paper 10/90, `freq >= 1`):** lost in 0003, and the frozen 10/90 with `freq > 1` lost again in 0004. Revisit it only as a control: a fixed split near the settled value, with the ghost pinned.
- **ARC-style two-ghost adaptation:** lost in 0003. Its extra ghost traffic pushed the split in harmful directions. Dropped.
- **Ghost structure:** the fingerprint ring works as a policy knob as well as plumbing. Its holes shorten the ghost's effective memory, which helped slightly. Ghost size and contents feed the adaptation directly.
- **CLOCK main queue:** deeper frequency caps help hit-rate but lengthen drains. Any cost to `put` from the main walk shows up in `get` too.
