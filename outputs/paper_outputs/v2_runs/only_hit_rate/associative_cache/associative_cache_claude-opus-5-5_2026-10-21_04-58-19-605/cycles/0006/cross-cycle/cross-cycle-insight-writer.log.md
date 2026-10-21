# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:16:43+00:00

## timeline

### 09:17:02 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Low protection on insert.** This has been the main hit-rate lever on workload-a since cycle 0001, and it worked on both the LRU and CLOCK lines. A newcomer has to show reuse before it gets protection. Strengthening this filter won or helped in cycles 0001–0003.\n- **The filter should be strict, but not maximal.** The best setting leaves a newcomer a little over one miss of grace: 1/8 depth on LRU, and hand-on-newcomer on GCLOCK. Removing the last buffer slot (pure LIP) hurt.\n- **Capped ghost admission.** A key that recently returned gets a small, bounded reward: counter 1 on CLOCK, or a slightly deeper insert on LRU. This helped on both lines (cycles 0004–0005). How big the reward is matters as much as the ghost signal itself.\n- **Exact LRU ordering is not needed.** In-place CLOCK/GCLOCK matches or comes close to shift-based LRU on hit rate once its newcomer filter matches, so its cheaper hits come at no hit-rate cost.\n\n## Recurring failures\n\n- **Generous rewards for returning keys.** Sending ghost hits straight to MRU (cycle 0002) weakened the newcomer filter and lost hit rate.\n- **Extra frequency-based protection on top of the filter.** This was tried three times and gained little or nothing: victim choice at the eviction end, SLRU-style two-step promotion, and frequency beyond the filter. The filter already removes most keys that are hit only once. GCLOCK's counter helped only against plain CLOCK, before the filter was tuned.\n- **Newcomer-leniency duels.** The cycle 0005 duel learned from its own outcome. The cycle 0006 retry used fixed DIP-style leaders and still lost. Lenient leaders cost hits just by running. This line is closed on workload-a.\n- **Moving the ghost ring away from its current settings.** Larger rings (16) diluted the signal. A smaller ring (2) lost slightly. Filtering what enters the ring removed useful hits on LRU. Ring size and ring admission are locally optimal at 4 with no admission filter.\n- **Per-item wall-clock timestamps.** They add nothing on the eval path, so this direction was dropped.\n- **Single-knob tweaks on workload-a.** Hit rate has looked saturated since cycle 0005. Recent wins and losses are small and may be within noise.\n\n## Open questions\n\n- **Missing measurements.** Per-op-time and metadata-size have never been recorded in any cycle. Every claim about latency or memory is untested, and candidates with equal hit rates cannot be ranked. These include the in-place CLOCK line against shift-based LRU, the ring scans on misses, and the 2- or 3-entry rings.\n- **Generalisation.** Does strict filtering plus a capped ghost counter hold up on a second trace or cache size, especially one with longer reuse distances? It may be over-fitted to workload-a.\n- **Main line.** Which line should lead once cost is counted: CLOCK+ghost or 1/8 LRU+ghost?\n- **Ring size of 3.** Does a 3-entry ghost ring match 4? This only matters once memory and miss-path cost are measured.\n- **Last untested hit-rate lever.** On the LRU line, how deep should a ghost hit be inserted?\n- **Concurrency cost.** Would a shared global atomic clock or stamp hurt multi-threaded throughput?\n\n## Per-policy notes\n\n- **Shift-based LRU, insertion-depth line.** Results by depth: midpoint, then 1/4, then 1/8 each improved, with smaller gains each step. 1/8 is the peak and pure LIP regressed. Ghost admission at index 3 gave a small gain, which may be noise. Filtering what enters the ghost ring hurt, because aged hit keys are the population the ghost reward is meant to rescue. Each miss shifts elements, and that cost has never been measured.\n- **CLOCK/GCLOCK.** Plain CLOCK beat LRU. A saturating counter (GCLOCK) improved on CLOCK. Resting the hand on the newcomer closed most of the gap to 1/8 LRU. A 4-entry ghost ring admitting at counter 1 (cycle 0004) is the strongest setup tested and is the settled shape. Rings of 2 and 16, and both leniency duels, all lost.\n- **Ghost rings in general.** Lessons do not carry over cleanly between policies, because each line's ring captures a different reuse-distance population. Validate each ghost rule separately on each line.\n- **Item-record size (u32 logical stamps).** Hit rate stayed the same as the baseline. The size and latency benefits were never measured.\n"
}
```

### 09:17:09 chat

I've written `cross-cycle-insights.md` from scratch using only the six-cycle ledger, with the four required sections in order. Per-op-time and metadata-size were never recorded in any cycle, so every comparison below is on workload-a hit rate only.

- **Recurring wins:**
  - Making new keys prove reuse before they get protection has been the main lever since cycle 0001.
  - That filter should be strict but leave a newcomer a bit more than one miss of grace: 1/8 insert depth on LRU, or leaving the GCLOCK hand on the newcomer.
  - Giving recently returning keys a small, capped bonus (a "ghost" ring) helped on both LRU and CLOCK.
  - In-place CLOCK/GCLOCK comes close to LRU on hit rate.
- **Recurring failures:**
  - Sending returning keys straight to most-recently-used.
  - Frequency-based protection on top of the newcomer filter, which gave little or nothing three times.
  - Duels that switch between strict and lenient grace for newcomers.
  - Moving the ghost ring away from 4 entries, or filtering which keys can enter it.
  - Per-item wall-clock timestamps.
  - Single-setting tweaks on workload-a, where hit rate looks saturated.
- **Open questions:**
  - Getting per-op-time and metadata-size recorded, which is needed to rank candidates with equal hit rates.
  - Testing on a second trace or cache size.
  - Choosing between the CLOCK+ghost and LRU+ghost lines.
  - Whether a 3-entry ring matches 4.
  - How deep ghost hits should be inserted on LRU, the last untested hit-rate setting.
  - The multi-threaded cost of a shared clock.
- **Per-policy notes:** these cover the LRU insert-depth line, the CLOCK/GCLOCK line, and the smaller-item-record variant. They also note that ghost-ring results on one policy don't reliably carry over to the other.


---

- finished: 2026-10-21T09:17:09+00:00
- duration: 25.83s
- status: ok
- input_tokens: 4
- output_tokens: 2423
- cache_read_tokens: 38760
- cache_write_tokens: 11842
- cost_usd: 0.15169
