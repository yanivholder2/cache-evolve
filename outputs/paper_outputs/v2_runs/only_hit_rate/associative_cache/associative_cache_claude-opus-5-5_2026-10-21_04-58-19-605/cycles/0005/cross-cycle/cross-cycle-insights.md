## Recurring wins

- **Low protection for newcomers is the main lever on workload-a (all 5 cycles).** New keys enter near the eviction end (or with a cleared or low counter) and earn protection only when they are reused. This beats keeping exact recency order. It has worked on both shift-based LRU and in-place CLOCK/GCLOCK.
- **The filter should be strict but not total.** Hit rate rose as the filter got stricter, up to about one slot (one miss) of grace: 1/8 depth on LRU, and the hand resting on the newcomer on GCLOCK. Pure LIP went past the peak.
- **Capped ghost admission helps.** A small, recent ghost ring that gives a returning key only a little extra grace improved hit rate on GCLOCK (Cycle 0004) and, by a smaller margin, on the 1/8 LRU line (Cycle 0005). It carries over between policy lines.
- **The size of the reward matters as much as the signal that triggers it.** The same signal (a ghost hit or a frequency count) helps when its reward is small and bounded, and hurts when the reward is a full promotion.

## Recurring failures

- **Frequency-based protection beyond the newcomer filter (Cycles 0002–0004).** This covers frequency-aware victim choice at the eviction end and SLRU-style promotion by frequency. The gains were at or near noise level, because the strict newcomer filter already removes keys that are used only once. GCLOCK counters helped plain CLOCK only before its newcomer filter was fixed.
- **Generous ghost rewards.** Sending ghost hits straight to MRU (Cycle 0002) lowered hit rate. Growing the ghost ring (from 4 to 16) diluted the signal and pushed out valuable residents (Cycle 0005).
- **Overshooting on a tuning axis.** Insert depth (pure LIP) and ghost ring size both have an interior peak. Pushing either one to the extreme regressed.
- **Adaptive controllers that learn from their own outcome.** The strict-versus-lenient duel locked itself into strict mode, so leniency was never actually tested (Cycle 0005).
- **The evaluation gap that never closed.** Per-op-time and metadata-size have not been recorded in any cycle. Every claim about latency or memory is untested, even though most winning variants added miss-path work: shifts, ghost scans, hand sweeps, per-bucket state.

## Open questions

- Once latency and memory are measured, which line should be the primary one: in-place GCLOCK with ghosts, or 1/8-depth LRU with ghosts? The ranking is currently based on hit rate alone.
- Do the strict filter and the small ghost ring generalise to other traces, longer reuse distances, or other cache sizes? Or are they over-fitted to workload-a at one size?
- Would a smaller ghost ring (2–3 entries) match or beat 4, while also cutting miss-path and memory cost?
- Would a properly built set-dueling controller (DIP-style leader buckets with an unbiased negative signal) ever choose leniency?
- Are the latest small margins above run-to-run noise? Hit rate on workload-a looks close to saturated.

## Per-policy notes

- **Shift-based LRU (insertion depth):** depth 1/2 → 1/4 → 1/8 gave shrinking gains, and index 0 regressed, so depth tuning is done. Capped ghost admission (a ghost hit enters slightly deeper than a normal newcomer) adds a small gain. Each miss pays for element shifts, a cost that has not been measured.
- **CLOCK / GCLOCK:** cleared bits on insert beat LRU even though CLOCK was designed for latency. Saturating counters helped over a single bit. The real gap to the LRU line was where the hand rests, not frequency. With the hand resting on the newcomer plus a 4-entry capped ghost ring, this is the strongest line by hit rate. Its no-shift hits may give it an edge on latency once latency is measured.
- **Item metadata:** replacing the wall-clock timestamp with a logical stamp kept hit rate unchanged. Shrinking the item record looks safe, but no size or throughput data exists yet, including any contention on a shared atomic clock.
