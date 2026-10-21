## Recurring wins

- **Strict newcomer filter is the dominant lever (cycles 1–4).** New keys enter with low protection and earn it only on a second hit. This improved hit rate on both shift-based LRU and in-place CLOCK. Stricter kept paying off until about one miss of grace; pure LIP went too far.
- **Capped ghost admission (cycles 4–5).** A small recent-eviction ring that gives returning keys a single extra unit of protection helped on both the CLOCK and LRU lines. Cycles 2, 4 and 8 point to the same rule: the size of the reward matters as much as the signal, so rewards must stay small for ghost returns and for first hits alike.
- **Ghost ring as a membership test (cycle 7).** Storing compact fingerprints instead of full keys left hit rate unchanged. This is the only cost-reducing change confirmed so far on the settled design.
- **Settled best shape: cycle/0007/v001.** CLOCK/GCLOCK with the hand resting on the newcomer, a +1 hit step, a counter ceiling of 3, and a 4-entry unfiltered fingerprint ghost ring. Every knob on it has been probed in both directions and peaks at its current setting.

## Recurring failures

- **Frequency-based protection beyond the newcomer filter (cycles 3, 4, 7).** This covered victim choice at the eviction end, SLRU-style promotion, and a broad miss-frequency admission sketch. It added nothing, because the filter already removes keys used only once. The broad sketch actively hurt: it saturated and in effect switched the filter off. Signals that grant a reward must stay selective.
- **Generous ghost rewards and wide rings (cycles 2, 5).** Promoting ghost hits to MRU, or growing the ring to cover longer reuse distances, diluted the signal and pushed out valuable residents.
- **Ghost-ring admission filters (cycles 6, 8).** Excluding never-hit keys (on LRU) or hit keys (on CLOCK) both lost hit rate. The unfiltered ring is already complete.
- **Newcomer-leniency duels (cycles 5, 6).** Self-referential controllers learned from their own outcome and stayed stuck in strict mode. Fixed DIP-style leaders removed that bias, but lenient grace still lost and the leaders cost hits. This thread is closed on workload-a.
- **Single-knob hit-rate tweaks after cycle 5.** Three cycles in a row produced no gain. The workload-a hit-rate axis is saturated for this design family.
- **Persistent evaluation gap (every cycle).** Per-op-time and metadata-size have never been recorded. Cost-saving variants (fingerprints, ceiling 2, smaller rings, cheaper hits) are invisible to the evaluation, and miss-path costs added by the winners have never been measured.

## Open questions

- Once per-op-time and metadata-size are measured, which shape wins: the CLOCK+ghost line, the 1/8 LRU+ghost line, a 3-entry fingerprint ring, or ceiling 2?
- Does the strict one-miss filter plus a 4-entry ghost ring generalise to a second trace or cache size, especially one with longer reuse distances? Or is it over-fitted to workload-a?
- Are the small losses and wins since cycle 5 above run-to-run noise?
- Does fingerprinting work equally well on the LRU line's ghost ring?
- Would a frequency-based admission signal with bounded false-positive error ever add anything beyond the exact ghost ring?

## Per-policy notes

- **Shift-based LRU (midpoint/deep insertion):** insert depth peaked at 1/8; pure LIP regressed. Ghost hits inserted at 1/4 depth help slightly, and deeper insertion (3/8) does not. Ghost admission filtering hurts. All knobs are bracketed, so keep this as a reference line only. Shifts on every miss carry an unmeasured latency cost.
- **CLOCK / GCLOCK:** the gap to LRU closed once the hand rested on the newcomer (cycle 3). A capped ghost ring then gave it the lead (cycle 4). Ring size peaks near 4, with both 2 and 16 losing. Hit step +1 with ceiling 3 is optimal. In-place hits make it the likely latency winner, but that is unverified.
- **Ghost mechanisms in general:** lessons do not transfer cleanly between lines, because each line's ring captures a different population (long-reuse keys on CLOCK, keys just past the bucket on LRU). Check each rule on each line separately.
- **Wall-clock timestamps:** they have no effect on eviction decisions, so smaller item records (logical stamps) are safe for hit rate. A shared atomic clock's multi-threaded cost is untested.
