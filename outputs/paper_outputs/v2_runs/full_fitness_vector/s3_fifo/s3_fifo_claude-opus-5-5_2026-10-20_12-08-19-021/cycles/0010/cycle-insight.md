# Cycle 0010 Insight

## Cohort outcome

This cycle had one depth-lever retry and two single-lever policy probes:

- **H01** went back to linear decrement on the 3-bit `MAX_FREQ = 7` counter and capped each `evict_main` at a fixed rotation budget of 32. Its parent was 0009 v001.
- **H02** reset a graduate's freq to 0 when it moves from small into main. Its parent was 0008 v001.
- **H03** lowered the upper clamp on `small_size` from `capacity/2` to `capacity/4`. Its parent was 0008 v001.

All three are Winners. v001 is the only one that moved hit-rate: it beat its parent and held `put` steady. It fell short of 0008 v003's level, and its `get` got worse even though the `get` code did not change. v003 and v002 left hit-rate exactly where it was, so both are null results that made the cut on rank.

## Patterns

- **Bounding the drain work separates retention depth from drain cost.** Cycle 0009 found that hit-rate comes from how many laps a key survives in main. Linear decrement plus a per-call rotation budget brings most of those laps back without the `put` tail that 0008 v003 paid. Unlike halving, the budget does not trade one for the other. Whatever gain is still missing likely comes from forced releases once the budget is spent.
- **Both bounds of the split are now ruled out as hit-rate levers on workload-a.** The 0005 floor did matter. The upper clamp, however, never binds: on workload-a, small appears to only shrink from its starting quarter, so the ceiling is dead code. This comes from v003's reasoning and is not yet confirmed by split-report data.
- **The credit a graduate brings from small doesn't matter.** Hot graduates get re-hit in main and saturate again before they reach the tail. Cold graduates' leftover laps are too few to change any eviction. Resetting freq at graduation gives the original S3-FIFO semantics at no hit-rate cost, but it is not a lever.
- **Latency drift went both ways in the same cohort.** v002 slowed on `get` and `put`, and v003 got faster tails. In both, the hot-path code was identical to the parent's. So v001's `get` regression can't be blamed on its eviction change without a same-run benchmark.
- **The split reports are still unreported.** This is the tenth cycle in a row. `forced_evictions`, `ceil_blocks`, `settled_small` and `main_rotations` were all added or requested, and none reached the writers.

## Open questions

- How often does the rotation budget bind? Would a larger budget (e.g. 64) close the rest of the gap to 0008 v003 without bringing back the `put` tail?
- Is v001's `get` regression real, perhaps from a different resident set changing probe or cache-line behaviour, or is it drift?
- Does small ever grow above its initial quarter on any workload? `ceil_blocks` and `max_small` from the Zipf sweeps would show it directly.

## Continuation

Adopt 0010 v001 as the parent: linear decrement, `MAX_FREQ = 7` packed into 3 bits, and a bounded `evict_main`. The next depth step is to tune the budget size, guided by `forced_evictions`. Drop all clamp tuning in either direction, and drop changes to graduate credit. v003 suggests the split's movement or the ghost size as the next lever, but triggers (0006, 0007) and every ghost probe (0002, 0005, 0008, 0009) have already failed, so treat that as low-priority. The loop infrastructure still has to benchmark each parent and child in the same run and get the `h01_split_report` stdout to the insight writers. No latency attribution in this cycle is trustworthy until it does.
