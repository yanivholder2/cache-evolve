# Cycle 0007 Insight

## Cohort outcome

All three variants used 0006 v002 as their parent. Two were policy-neutral trims and one changed policy:

- **H01** moved `freq` into a parallel `freqs` array, which shrinks `Entry` to 8 bytes.
- **H02** doubled the ghost-hit shrink step, with the ghost pinned.
- **H03** skipped the `freq` store on `get` when the counter is already saturated.

All three are Winners, but none improved without a regression. v001 ranks first on a real per-slot `metadata-size` drop. v003 and v002 made the cut on rank: hit-rate was flat across the whole cohort, and every variant regressed on latency.

## Patterns

- **The benchmark baseline drifted this cycle.** `put` slowed in all three variants, including v003, whose `put` path is byte-identical to the parent's. So this cycle's latency deltas can't be credited to the code, in either direction. That includes v001's `get` loss, even though it has a plausible cause: a second cache line on every hit.
- **Splitting hot fields from each other is a different trade from splitting cold fields out.** Moving the key out (0006) helped `get`. Moving `freq` out splits two fields that every hit reads together. The metadata saving is real, but at this capacity the slab already fits in cache, so there was little locality to win back.
- **Neither trigger ratio is a lever on workload-a.** Cycle 0006 found the grow-small side inert, and the ghost-hit shrink side is now inert too. If the split responds to anything, it is the bounds (the `capacity/64` floor) or the ghost sizing, not how fast it moves between them.
- **Store-elision micro-trims on `get` are below what this benchmark can resolve.** At this measurement noise, a single-threaded L1-resident store costs too little to show up.
- **`small_size` is still unreported.** This is the seventh cycle in a row. All three writers said the split-report output was not in the artifacts they could reach. The variants did generate it, but it never reaches the insight writer.

## Open questions

- Is v001's `get` regression real, or drift? Only a same-run parent-vs-child benchmark can tell.
- Can `freq` be packed into spare bits next to the value or the index entry? That would keep the 8-byte saving without a second access per hit.
- Does `settled_small` sit at the floor under both shrink steps (`floor_blocks`)? If so, the floor is the binding limit.
- Is the pinned ghost capacity, rather than the split bounds, what caps hit-rate now?

## Continuation

Keep 0006 v002 as the latency parent, and carry v001's `freqs` split only as a metadata-size branch until a same-run re-baseline clears its `get` cost. Prefer a bit-packed `freq` over a separate array. Drop trigger-step tuning on both sides, and drop `get` store-elision. The next policy lever should be the ghost capacity (held relative to the initial main size) or a test of whether the floor binds. Before trusting any latency claim, the loop must (1) benchmark parent and child back-to-back in the same run and (2) copy the `h0x_split_report` stdout into the eval artifacts the insight writers can read.
