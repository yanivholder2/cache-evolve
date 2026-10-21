## Recurring wins

- Changes that only remove work or shrink layout, with no new hot-path logic, are the most reliable gains. Fast hashing (cycle 1) and compact `u32` links (cycle 2) both kept behaviour unchanged and improved latency or memory.
- Hit rate moves only when the frequency estimates get more accurate. Conservative update (cycle 3) is the only hit-rate gain so far. Every other admission, structure or victim change left hit rate flat or made it worse.
- The lineage has a clean chain of bases: fast hash → compact links → conservative update. Each step kept the previous wins.

## Recurring failures

- Adding resident-list structure (window + SLRU in cycle 1, LRU-end victim sampling in cycle 2) costs latency and memory and gives no hit-rate gain. The sketch signal is the bottleneck, not the list structure.
- Changing the admission rule or counter resolution on its own fails. Aging with a coarser cap created more ties and rejected good newcomers (cycle 2). Letting newcomers win ties also hurt (cycle 3), so the strict `>` rule is not the problem.
- Sketch layouts that break row independence (the blocked/cache-line sketch in cycle 3) inflate estimates and hurt admission. The sketch already fits in cache, so layout has nothing to win there.
- Any per-op branch, bookkeeping step or extra hash shows up in latency right away (aging reset tracking, extra hashing per miss, the two-pass conservative-update increment).
- Per-op timings have drifted for unexplained environmental reasons in two cycles running. Small latency deltas between variants can't be trusted unless the variant and its parent are benchmarked side by side.

## Open questions

- Does aging or a doorkeeper help once conservative update limits overestimation? Aging has only been measured once (cycle 2), before conservative update existed. The cycle-1 attempt never ran because of an infrastructure failure.
- Can a wider or deeper sketch that keeps row independence add to the accuracy gain at an acceptable metadata cost?
- Can the conservative-update increment become single-pass or stay in registers, so `get` recovers its latency?
- How close is workload-a to the hit-rate ceiling for a TinyLFU-family design?
- Is the unkeyed fast hasher safe against adversarial or collision-prone keys?
- Was cycle 2's latency regression for compact links real or just noise? This was never confirmed side by side.

## Per-policy notes

- TinyLFU (plain LRU + CountMin admission): this is the current line. The best base is fast hash + compact links + conservative update. The levers that work are estimate accuracy and hot-path cost.
- W-TinyLFU (window + SLRU): tried once with a ~1% window, it lost on memory and latency with almost no hit-rate gain. Worth revisiting only after the frequency signal improves, and only with low per-hit relink cost.
- Sampled/frequency-aware eviction: entries near the LRU end have nearly equal estimates, so sampling adds hashing cost for no benefit. Drop it.
- Aging (Caffeine-style reset): it hurt on its own with a coarse cap and strict ties. It is untested with conservative update or a doorkeeper.
