# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:33:17+00:00

## timeline

### 14:33:39 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0007)\n\n## Recurring wins\n\n- **The adaptive small/main split is the durable policy core.** It came in at 0001 and has beaten every challenger since: ARC-direction signals, the published static 10/90 split, a frozen 10/90 split with the incumbent graduation rule kept, and ghost resizing. It is a package (adaptation plus `freq > 1` graduation plus clamps), and it absorbs most perturbations to other levers.\n- **The adaptation's bounds are the only policy lever that has moved hit-rate since 0001.** Lowering the small-queue floor to `capacity/64` gave a small gain (0005). The gain reproduced exactly on a different storage layout (0006).\n- **Removing O(n) and machine work gives real latency wins.** These include hashed indexing instead of linear scans (0002), the fingerprint ring ghost with no clones or compaction (0003), and replacing modulo with a compare plus inlined eviction on `put` (0004). Every credible latency gain traces to a specific removed piece of work on the measured path.\n- **Splitting hot and cold data with a narrow index helps `get`.** The key moved out to a parallel array behind the `(K, u32)` index (0006). The `get` path should touch only the fields it reads together (`value`, `freq`).\n- **Policy and layout levers are independent and stack freely across lineages.** Policy-neutral refactors stayed policy-exact, and policy gains carried over unchanged across layouts.\n\n## Recurring failures\n\n- **Strengthening the movement or ghost path backfires.** This held for ghosting main victims (0002), extra ghost traffic or ARC two-ghost signals (0003), and readmit-with-`freq` credit (0005). Extra ghost hits push the split around or keep one-time-reuse keys in main for too long.\n- **Neither trigger rate is a lever.** Damping grow-small (0006) and doubling the ghost-hit shrink step (0007) were both inert on hit-rate. Tuning how fast the split moves does nothing; only where its bounds sit matters.\n- **Static splits lose,** both the wholesale replacement (0003) and the one-lever version (0004). Ghost-size tuning lost too: shrinking the ghost was absorbed by the clamps.\n- **Two-lever and coupled variants can't be read.** Variants that changed the split together with graduation or ghost size (0003, 0004) could not attribute their losses. Ghost size kept getting tied to main size by accident.\n- **Wide or inline probe-table layouts slow `get`.** Inline payloads gave buckets whose size is not a power of two (0005 and 0006). Splitting fields that every hit reads together into separate arrays (`freqs`, 0007) costs a second access for little locality gain at cache-resident capacity.\n- **Micro-trims below the benchmark's noise floor** (store elision on `get`, 0007) can't be resolved by this benchmark.\n- **Rank-only Winners.** In most cycles at least one \"Winner\" lost to its parent on both axes and got in only on cohort rank. Don't treat Winner status as evidence a lever works.\n\n## Open questions\n\n- **Where does `small_size` actually settle, and does the floor bind (`floor_blocks`)?** Seven cycles have asked. The split reports are generated but never reach the insight writers. The loop must copy that stdout into the eval artifacts.\n- **Is ghost capacity, held relative to the initial main size, now what caps hit-rate?** It was only ever tested together with other levers or absorbed by the clamps. Test it with the clamps fixed.\n- **Is a static split at the settled value as good as adaptation,** with ghost size pinned at the parent's value? This is still the clean test that separates adaptation from ratio.\n- **Can `freq` be bit-packed next to the value or index entry?** That would keep the 8-byte `Entry` without a second access per hit.\n- **Would a niche-free `keys: Vec<K>` with liveness tracked through `free` really shrink the per-slot footprint?** `metadata-size` alone overstates this saving.\n- **Conditional readmit credit** (only on a second ghost hit, or only while small is at its floor) is untested.\n- **Several latency deltas are still unverified:** the `get` cost of `freqs`, the `put` cost of a longer main, and others. These need same-run parent-versus-child benchmarking.\n\n## Per-policy notes\n\n- **S3-FIFO (adaptive split lineage):** current parent lineage is 0006 v002, with the `capacity/64` floor, a narrow `(K, u32)` index, the hot/cold split with keys out of the slab, and the fingerprint ring ghost. Carry 0007 v001 (`freqs` split) only as a metadata-size branch.\n- **S3-FIFO ghost:** it acts as a policy knob, not plumbing. Its effective memory, including hole handling in the ring, feeds the split adaptation directly. Pin its size whenever another lever is under test.\n- **Metrics caveats:** `metadata-size` counts only `size_of::<Entry>`. It hides all ghost heap and parallel-array costs, so treat it as a proxy for the hot array. `get` latency often moves when the `get` code is byte-identical to the parent's, and p99 is especially noisy. Baseline drift (0007) can shift every variant at once. Credit latency only to changes on the path that was measured, and only with a same-run re-baseline.\n- **Infrastructure:** build failures (Maven 429s in 0001) can reject variants for reasons unrelated to their code. Re-submit those variants rather than counting them as negative results.\n"
}
```

### 14:33:49 chat

I wrote `cross-cycle-insights.md` in the cycle-0007 cross-cycle directory. It has the four required sections in order and was built only from the seven-cycle ledger.

**Recurring wins**
- The adaptive small/main split has beaten every challenger since cycle 0001. That includes ARC-style signals, static splits and ghost resizing.
- The small-queue floor is the only policy lever that has moved hit-rate since then. Lowering it to `capacity/64` gave a small gain, and the gain held on a different storage layout.
- Every credible latency gain came from removing specific work: hashed indexing instead of linear scans, the fingerprint ring ghost, and a leaner `put` path.
- Moving cold fields like the key out of what `get` reads, behind a narrow index, made `get` faster.
- Policy changes and layout changes don't interfere, so wins from different lineages can be combined.

**Recurring failures**
- Making the ghost path stronger backfired every time it was tried.
- Tuning how fast the split moves did nothing on either side; only where its bounds sit matters.
- Static splits lost, and so did ghost-size tuning.
- Variants that changed two levers at once couldn't be read, because nobody could tell which lever caused the loss.
- Wide or inline probe tables slowed `get`, as did splitting `freq` into its own array.
- Micro-trims were too small for the benchmark to measure.
- Several "Winners" got in only on rank within their cohort, so Winner status alone isn't evidence a lever works.

**Open questions**
- Where `small_size` actually settles is still unknown after seven cycles. The split reports are generated but never reach the insight writers, so the loop needs to copy that output into the artifacts they can read.
- Ghost capacity, tested on its own with the clamps fixed, may be what limits hit-rate now.
- A static split at the settled value, with ghost size pinned, would separate the effect of adaptation from the split ratio.
- Other untested ideas: packing `freq` into spare bits, a niche-free keys array, and readmit credit that only applies under certain conditions.
- Several latency changes need parent and child benchmarked back-to-back in the same run before they can be trusted.

**Per-policy notes**
- The current parent is cycle 0006's v002. Cycle 0007's v001 (`freq` in its own array) should be kept only as a branch for reducing `metadata-size`.
- The ghost works as a policy setting, so its size should be fixed whenever another lever is being tested.
- `metadata-size` hides ghost memory and the cost of parallel arrays, so it overstates savings.
- `get` latency moves even when the `get` code hasn't changed, so latency should only be credited to code that actually changed on the measured path.
- Variants rejected because of build failures, like the Maven errors in cycle 0001, should be re-submitted rather than counted as losses.


---

- finished: 2026-10-20T14:33:49+00:00
- duration: 32.53s
- status: ok
- input_tokens: 4
- output_tokens: 2998
- cache_read_tokens: 40426
- cache_write_tokens: 13791
- cost_usd: 0.178389
