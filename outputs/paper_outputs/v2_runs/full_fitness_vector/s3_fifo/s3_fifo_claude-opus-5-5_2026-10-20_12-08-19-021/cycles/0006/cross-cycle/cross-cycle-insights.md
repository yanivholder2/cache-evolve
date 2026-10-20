# Cross-Cycle Insights

## Recurring wins

- **The adaptive small/main split has beaten everything for six cycles in a row.** It won in cycle 0001. Since then it has outlasted the ARC-style two-ghost signals, the paper's static 10/90 split, and a frozen split that kept the incumbent graduation rule. The adaptation itself earns its margin; it is not just an escape from a bad 25/75 start.
- **The adaptation's lower clamp is the only policy lever that has added hit-rate.** Lowering the small floor from `capacity/16` to `capacity/64` gave a small but real gain. It reproduced exactly on a different storage layout.
- **Latency wins come from removing real work in the code path:**
  - linear scans replaced by hashed indexing (the biggest gain);
  - key clones and `retain` compaction removed from the ghost (fingerprint ring);
  - the ring's modulo wrap replaced by a compare, and the eviction functions inlined.
- **On the `get` path, splitting hot and cold data pays off.** Move cold fields such as the key out of the array `get` reads, keep the narrow `(K, u32)` index, and keep `get` reading only `{value, freq}`. This made `get` faster with no regressions.
- **Policy and layout levers are independent and stack.** Changes to one did not disturb the other, so winners from different lineages can be combined freely.
- **Variants that change one lever at a time, on the current incumbent, are the only ones that give readable results.**

## Recurring failures

- **Making the ghost path stronger backfires.** Three attempts lost hit-rate: ghosting main victims (cycle 0002), main-ghost hits that bypass `small` (cycle 0003), and blanket `freq` credit on readmission (cycle 0005). Readmission straight to main is already the reward, and extra ghost traffic pushes the split in a bad direction.
- **Movement-rule tweaks get absorbed by the adaptation.** A looser graduation rule was flat. The paper's `freq >= 1` graduation and freq reset lost when swapped in alongside the split.
- **Static splits lose to the adaptation.** This held both times it was tested.
- **Ghost-size changes and grow-small damping barely move hit-rate.** The clamps soak up ghost-length changes, and damping the grow-small trigger was neutral.
- **Inline-payload probe tables slow `get`.** Wide buckets that are not a power of two cost more than the dependent load they save. This showed up in both cycles that tried it.
- **Two-lever variants can't be attributed.** Changing the split also changed the ghost size, through `ghost_cap` being tied to main size. That confound appeared in cycles 0003 and 0004.
- **Some claimed levers were miscounts.** A "redundant hashing" premise turned out wrong because the parent already hashed and cloned as little as the refactor. Count the parent's work before claiming a lever.
- **Infrastructure:** Maven HTTP 429s once failed builds, which wasted a cohort slot.

## Open questions

- **Where does `small_size` actually settle?** Six cycles have asked for this, and the instrumentation exists (`h01_split_report`, `h03_split_report`). No settled value has ever reached an insight. Nobody knows how often the floor binds, or whether losing variants shifted the split.
- **Does the ghost-hit shrink side set the balance point?** The grow-small trigger looks inert. The next policy probes should be on the shrink side, one lever at a time with the ghost size pinned.
- **Does conditional readmit credit behave differently?** Examples: credit only on a second ghost hit, or only while `small` is at its floor.
- **Would a static 25/75 split with the ghost pinned at the parent's size match the adaptive split?** This would isolate the adaptation from the split ratio and the ghost size.
- **Would a niche-free `keys: Vec<K>`, with liveness tracked through `free`, cut the real per-slot footprint and keep the `get` gain?**
- **Is a true single-probe miss path (raw-entry or precomputed hash) a real `put` lever?**
- **Which latency moves are real?** `get` and p99 move noticeably even when the code is byte-identical. A same-run re-baseline against the parent is still needed.
- **Does the delayed `Drop` of stale values matter when `V` is non-trivial?**

## Per-policy notes

- **S3-FIFO incumbent (adaptive split, `freq > 1` graduation, fingerprint-ring ghost, `capacity/64` floor, split hot/cold arrays):** This is the current parent lineage. Each adopted parent kept the policy unchanged and improved latency or layout.
- **ARC-style two-ghost adaptation:** It lost to the lopsided incumbent adaptation. Its two levers were never untangled. Not worth resuming wholesale.
- **Paper-faithful static S3-FIFO (10/90 split, `freq >= 1` graduation):** It lost on hit-rate, and it also lost with the incumbent graduation rule kept. Its static split loses to the adaptation on workload-a.
- **Ghost structure:** The ghost behaves like a policy knob, not plumbing. Its effective memory, including the holes left in the ring, feeds the split adaptation. A shorter ghost was roughly neutral to slightly favourable. Adding ghost traffic hurt.
- **Metrics caveats:**
  - `metadata-size` counts only `size_of::<Entry>`. It misses ghost heap, side arrays and per-cache fields, so read it as a proxy for the hot array.
  - The headline latency tracks `get`, so a `put`-only refactor cannot improve the summary.
