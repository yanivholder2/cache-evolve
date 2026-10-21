## Recurring wins

- **Low protection on insert.** This has been the main hit-rate lever on workload-a since cycle 0001, and it worked on both the LRU and CLOCK lines. A newcomer has to show reuse before it gets protection. Strengthening this filter won or helped in cycles 0001–0003.
- **The filter should be strict, but not maximal.** The best setting leaves a newcomer a little over one miss of grace: 1/8 depth on LRU, and hand-on-newcomer on GCLOCK. Removing the last buffer slot (pure LIP) hurt.
- **Capped ghost admission.** A key that recently returned gets a small, bounded reward: counter 1 on CLOCK, or a slightly deeper insert on LRU. This helped on both lines (cycles 0004–0005). How big the reward is matters as much as the ghost signal itself.
- **Exact LRU ordering is not needed.** In-place CLOCK/GCLOCK matches or comes close to shift-based LRU on hit rate once its newcomer filter matches, so its cheaper hits come at no hit-rate cost.

## Recurring failures

- **Generous rewards for returning keys.** Sending ghost hits straight to MRU (cycle 0002) weakened the newcomer filter and lost hit rate.
- **Extra frequency-based protection on top of the filter.** This was tried three times and gained little or nothing: victim choice at the eviction end, SLRU-style two-step promotion, and frequency beyond the filter. The filter already removes most keys that are hit only once. GCLOCK's counter helped only against plain CLOCK, before the filter was tuned.
- **Newcomer-leniency duels.** The cycle 0005 duel learned from its own outcome. The cycle 0006 retry used fixed DIP-style leaders and still lost. Lenient leaders cost hits just by running. This line is closed on workload-a.
- **Moving the ghost ring away from its current settings.** Larger rings (16) diluted the signal. A smaller ring (2) lost slightly. Filtering what enters the ring removed useful hits on LRU. Ring size and ring admission are locally optimal at 4 with no admission filter.
- **Per-item wall-clock timestamps.** They add nothing on the eval path, so this direction was dropped.
- **Single-knob tweaks on workload-a.** Hit rate has looked saturated since cycle 0005. Recent wins and losses are small and may be within noise.

## Open questions

- **Missing measurements.** Per-op-time and metadata-size have never been recorded in any cycle. Every claim about latency or memory is untested, and candidates with equal hit rates cannot be ranked. These include the in-place CLOCK line against shift-based LRU, the ring scans on misses, and the 2- or 3-entry rings.
- **Generalisation.** Does strict filtering plus a capped ghost counter hold up on a second trace or cache size, especially one with longer reuse distances? It may be over-fitted to workload-a.
- **Main line.** Which line should lead once cost is counted: CLOCK+ghost or 1/8 LRU+ghost?
- **Ring size of 3.** Does a 3-entry ghost ring match 4? This only matters once memory and miss-path cost are measured.
- **Last untested hit-rate lever.** On the LRU line, how deep should a ghost hit be inserted?
- **Concurrency cost.** Would a shared global atomic clock or stamp hurt multi-threaded throughput?

## Per-policy notes

- **Shift-based LRU, insertion-depth line.** Results by depth: midpoint, then 1/4, then 1/8 each improved, with smaller gains each step. 1/8 is the peak and pure LIP regressed. Ghost admission at index 3 gave a small gain, which may be noise. Filtering what enters the ghost ring hurt, because aged hit keys are the population the ghost reward is meant to rescue. Each miss shifts elements, and that cost has never been measured.
- **CLOCK/GCLOCK.** Plain CLOCK beat LRU. A saturating counter (GCLOCK) improved on CLOCK. Resting the hand on the newcomer closed most of the gap to 1/8 LRU. A 4-entry ghost ring admitting at counter 1 (cycle 0004) is the strongest setup tested and is the settled shape. Rings of 2 and 16, and both leniency duels, all lost.
- **Ghost rings in general.** Lessons do not carry over cleanly between policies, because each line's ring captures a different reuse-distance population. Validate each ghost rule separately on each line.
- **Item-record size (u32 logical stamps).** Hit rate stayed the same as the baseline. The size and latency benefits were never measured.
