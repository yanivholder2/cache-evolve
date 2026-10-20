# Cross-Cycle Insights (through cycle 0009)

## Recurring wins

- **The adaptive small/main split is the core hit-rate asset.** It has beaten static splits three times: the paper's 10/90 split (0003), a frozen 10/90 split with the incumbent graduation rule kept (0004), and the original 25/75 split (0001). It also absorbs most changes to the other levers.
- **Hit-rate moves only through the policy's bounds or depth, never through trigger rates.** The only hit-rate gains after 0001 came from lowering the small floor to `capacity/64` (0005, a small gain) and from deeper retention through `MAX_FREQ` 7 (0008, a small gain at a latency cost).
- **Removing work gives real latency gains.** Swapping O(capacity) scans for hashed indexing (0002), using a fingerprint ring ghost instead of key clones (0003), and trimming the `put` path (0004) each traced to specific deleted work.
- **The `get` path should touch only memory it already reads.** Moving cold fields such as the key out of the hot array helped (0006). Bit-packing `freq` into the index bucket that `get` already probes kept the metadata saving without slowing `get` (0008). This rule has held for three cycles.
- **Policy and layout levers stack independently.** A policy gain reproduced exactly on a different layout (0006), and layout refactors left hit-rate unchanged every time they were tested.

## Recurring failures

- **Every change to the ghost or readmission path has lost hit-rate, whatever its direction.** Failed probes:
  - ghosting main victims (0002)
  - readmitting with `freq` credit (0005)
  - a full-capacity ghost (0008)
  - routing ghost hits through small (0009)
  - ARC-style two-ghost signals (0003)
  - halving the ghost, which did nothing (0004)

  Direct ghost-to-main readmission is what ties the split together, and it looks like a local optimum.
- **Tuning trigger rates does nothing on workload-a.** Damping grow-small (0006) and doubling the shrink step (0007) were both neutral.
- **Changing movement rules on top of the adaptation goes nowhere.** Loosening graduation was flat (0002), and replacing the policy wholesale lost (0003).
- **Variants that change two levers at once can't be read.** Coupling ghost size to the split (0003, 0004) or changing the split and graduation together (0003) left each loss unattributable.
- **Layouts that widen the probe or split hot fields hurt `get`.** Inlining the payload in the probe table was slower (0005, 0006). A separate `freqs` array added a second access on every hit (0007).
- **Probe-table internals are not the bottleneck.** A custom linear-probe table moved cost onto `put` without improving `get` p50 (0009). Micro-trims to `get` stores are below what the benchmark can resolve (0007).
- **Cheaper decay schedules give up the depth gain.** Halving drain cut the eviction cost but also removed the long retention that produced the hit-rate (0009).

## Open questions

- **Can `MAX_FREQ` 7 keep its linear retention with bounded eviction work?** Candidates are a fixed scan budget per eviction and decrementing a sampled subset.
- **Can a prefetch of the value cut `get` p50?** The prefetch would be issued from the probed bucket. Removing the second dependent load is the only remaining `get` lever.
- **Where does `small_size` settle, and does the `capacity/64` floor bind?** The `floor_blocks` count would answer the second part. This has gone unreported for nine cycles because split-report stdout never reaches the insight writers. It's an infrastructure gap, not a hypothesis gap.
- **Which latency deltas are real?** Every cycle has seen `get` move even when the `get` code was byte-identical to the parent's, and 0007 showed baseline drift across the whole cohort. Same-run parent-vs-child benchmarking is still missing.
- **Does `metadata-size` reflect real memory?** It counts only `Entry`. It misses ghost heap, side arrays and per-cache fields, so it should be read as a proxy for the hot array.

## Per-policy notes

- **Adaptive S3-FIFO (incumbent lineage):** current parent is 0008 v001. It uses hashbrown with a narrow `(K, u32)` index, `freq` packed into the index bits, an 8-byte slab `Entry` with keys split out, the `capacity/64` floor, decrement drain and direct ghost-to-main readmission. Its two resize triggers are lopsided, and the ghost-hit shrink side appears to set the balance point.
- **Static-split S3-FIFO (10/90 and 25/75):** it lost to the adaptive split every time it was tested. Don't retry it unless ghost size is pinned at the parent's value and only one lever changes.
- **ARC-style signals and two ghosts:** lost hit-rate in 0003. Dropped.
- **CLOCK drain in main:** retention depth (`MAX_FREQ`) is a real lever. Its cost lands on `put`, and through cache pressure in the mixed benchmark it also reaches `get`.
