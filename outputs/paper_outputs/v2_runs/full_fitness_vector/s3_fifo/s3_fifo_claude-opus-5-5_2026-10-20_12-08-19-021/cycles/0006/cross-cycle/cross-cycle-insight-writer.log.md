# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:26:32+00:00

## timeline

### 14:26:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **The adaptive small/main split has beaten everything for six cycles in a row.** It won in cycle 0001. Since then it has outlasted the ARC-style two-ghost signals, the paper's static 10/90 split, and a frozen split that kept the incumbent graduation rule. The adaptation itself earns its margin; it is not just an escape from a bad 25/75 start.\n- **The adaptation's lower clamp is the only policy lever that has added hit-rate.** Lowering the small floor from `capacity/16` to `capacity/64` gave a small but real gain. It reproduced exactly on a different storage layout.\n- **Latency wins come from removing real work in the code path:**\n  - linear scans replaced by hashed indexing (the biggest gain);\n  - key clones and `retain` compaction removed from the ghost (fingerprint ring);\n  - the ring's modulo wrap replaced by a compare, and the eviction functions inlined.\n- **On the `get` path, splitting hot and cold data pays off.** Move cold fields such as the key out of the array `get` reads, keep the narrow `(K, u32)` index, and keep `get` reading only `{value, freq}`. This made `get` faster with no regressions.\n- **Policy and layout levers are independent and stack.** Changes to one did not disturb the other, so winners from different lineages can be combined freely.\n- **Variants that change one lever at a time, on the current incumbent, are the only ones that give readable results.**\n\n## Recurring failures\n\n- **Making the ghost path stronger backfires.** Three attempts lost hit-rate: ghosting main victims (cycle 0002), main-ghost hits that bypass `small` (cycle 0003), and blanket `freq` credit on readmission (cycle 0005). Readmission straight to main is already the reward, and extra ghost traffic pushes the split in a bad direction.\n- **Movement-rule tweaks get absorbed by the adaptation.** A looser graduation rule was flat. The paper's `freq >= 1` graduation and freq reset lost when swapped in alongside the split.\n- **Static splits lose to the adaptation.** This held both times it was tested.\n- **Ghost-size changes and grow-small damping barely move hit-rate.** The clamps soak up ghost-length changes, and damping the grow-small trigger was neutral.\n- **Inline-payload probe tables slow `get`.** Wide buckets that are not a power of two cost more than the dependent load they save. This showed up in both cycles that tried it.\n- **Two-lever variants can't be attributed.** Changing the split also changed the ghost size, through `ghost_cap` being tied to main size. That confound appeared in cycles 0003 and 0004.\n- **Some claimed levers were miscounts.** A \"redundant hashing\" premise turned out wrong because the parent already hashed and cloned as little as the refactor. Count the parent's work before claiming a lever.\n- **Infrastructure:** Maven HTTP 429s once failed builds, which wasted a cohort slot.\n\n## Open questions\n\n- **Where does `small_size` actually settle?** Six cycles have asked for this, and the instrumentation exists (`h01_split_report`, `h03_split_report`). No settled value has ever reached an insight. Nobody knows how often the floor binds, or whether losing variants shifted the split.\n- **Does the ghost-hit shrink side set the balance point?** The grow-small trigger looks inert. The next policy probes should be on the shrink side, one lever at a time with the ghost size pinned.\n- **Does conditional readmit credit behave differently?** Examples: credit only on a second ghost hit, or only while `small` is at its floor.\n- **Would a static 25/75 split with the ghost pinned at the parent's size match the adaptive split?** This would isolate the adaptation from the split ratio and the ghost size.\n- **Would a niche-free `keys: Vec<K>`, with liveness tracked through `free`, cut the real per-slot footprint and keep the `get` gain?**\n- **Is a true single-probe miss path (raw-entry or precomputed hash) a real `put` lever?**\n- **Which latency moves are real?** `get` and p99 move noticeably even when the code is byte-identical. A same-run re-baseline against the parent is still needed.\n- **Does the delayed `Drop` of stale values matter when `V` is non-trivial?**\n\n## Per-policy notes\n\n- **S3-FIFO incumbent (adaptive split, `freq > 1` graduation, fingerprint-ring ghost, `capacity/64` floor, split hot/cold arrays):** This is the current parent lineage. Each adopted parent kept the policy unchanged and improved latency or layout.\n- **ARC-style two-ghost adaptation:** It lost to the lopsided incumbent adaptation. Its two levers were never untangled. Not worth resuming wholesale.\n- **Paper-faithful static S3-FIFO (10/90 split, `freq >= 1` graduation):** It lost on hit-rate, and it also lost with the incumbent graduation rule kept. Its static split loses to the adaptation on workload-a.\n- **Ghost structure:** The ghost behaves like a policy knob, not plumbing. Its effective memory, including the holes left in the ring, feeds the split adaptation. A shorter ghost was roughly neutral to slightly favourable. Adding ghost traffic hurt.\n- **Metrics caveats:**\n  - `metadata-size` counts only `size_of::<Entry>`. It misses ghost heap, side arrays and per-cache fields, so read it as a proxy for the hot array.\n  - The headline latency tracks `get`, so a `put`-only refactor cannot improve the summary.\n"
}
```

### 14:27:02 chat

I wrote `cross-cycle-insights.md` in the cycle 0006 cross-cycle directory. It uses the four required sections in order and is built only from the six-cycle ledger, with no numbers added.

The main points it carries forward:

- **What keeps winning:** The adaptive small/main split has beaten every challenger for six cycles. The only policy change that added hit-rate was lowering the small-queue floor from `capacity/16` to `capacity/64`. Latency gains came only from removing real work. Those were hashed indexing, a cheaper ghost, and keeping the fields `get` reads separate from the rest. Policy and layout changes stack without interfering.
- **What keeps failing:**
  - Making the ghost path stronger (three tries).
  - Tweaking the movement rules, which the adaptation absorbs.
  - Static splits.
  - Ghost-size and grow-small tuning.
  - Wide inline-payload index layouts.
  - Variants that change two things at once, so the result can't be pinned on either.
- **Biggest open gap:** Six cycles have asked where `small_size` settles, and no insight has ever reported a value. The other open threads are:
  - Policy probes on the ghost-hit side of the split, one change at a time with ghost size fixed.
  - A `keys` array that really cuts memory per slot.
  - A same-run latency re-baseline, because `get` and p99 move even when the code is unchanged.
- **Metric warning:** `metadata-size` counts only the main `Entry` struct and misses ghost memory and side arrays. The headline latency tracks `get`, so a change that only speeds up `put` can't improve it.


---

- finished: 2026-10-20T14:27:02+00:00
- duration: 29.84s
- status: ok
- input_tokens: 4
- output_tokens: 2614
- cache_read_tokens: 39328
- cache_write_tokens: 12728
- cost_usd: 0.161986
