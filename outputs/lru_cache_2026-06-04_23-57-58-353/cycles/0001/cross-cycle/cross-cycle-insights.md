# Cross-Cycle Insights

## Recurring wins

- Targeting the one-hit / reused distinction pays off: every variant that attacked scan-pollution at any stage improved hit rate over baseline LRU — none regressed.
- Acting at **insertion time** (routing new entries into a probationary tier before they can touch the protected working set) has proven more decisive than acting at eviction time alone.

## Recurring failures

- Eviction-time-only strategies carry a structural ceiling: CLOCK and fixed-window frequency scanning both improve hit rate but cannot prevent cold entries from entering the working set in the first place, so their gains are bounded.
- Fixed-size look-ahead windows (e.g., 8 candidates) leave entries just outside the window fully exposed regardless of their actual access frequency, capping improvement without adaptive sizing.

## Open questions

- Does adjusting the SLRU protected-to-probationary split ratio (currently 80/20) meaningfully shift the balance between scan resilience and working-set capacity on scan-heavy workloads?
- Can frequency-based demotion be layered *inside* the SLRU protected segment (evicting low-frequency protected entries back to probationary rather than strict LRU demotion) to compound gains?
- Does CLOCK's elimination of linked-list pointer maintenance translate to measurable throughput advantages under high concurrency or very large capacities, even where its hit-rate ceiling is lower?
- Would an adaptive frequency-window (growing beyond 8 candidates) close the gap with insertion-time strategies, or is the eviction-only placement the binding constraint regardless of window size?

## Per-policy notes

- **SLRU (v001):** Sole winner in cycle 1. Two-segment design (probationary + protected) defeats scan pollution at insertion time. Split ratio unexplored — a key tuning axis for next cycles.
- **CLOCK (v002):** Improved over baseline but trailed SLRU. Structural simplicity (no pointer maintenance) may yield concurrency or throughput wins not visible in single-threaded hit-rate benchmarks; worth revisiting in that context.
- **Frequency-window eviction (v003):** Gained versus baseline but plateaued due to fixed 8-slot window and eviction-time placement. The core idea (frequency as a promotion/demotion signal) is not exhausted — it may be more powerful inside SLRU's protected segment than as a standalone eviction policy.
