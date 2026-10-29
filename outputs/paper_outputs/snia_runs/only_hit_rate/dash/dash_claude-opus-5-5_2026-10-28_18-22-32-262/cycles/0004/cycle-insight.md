# Cycle 0004 Insight

## Cohort outcome

All three variants built on cycle-0003 v003 (`Back` promotion placement plus the two-hit gate). Each tried a different lever: v001 skipped the gate when the target has two free slots, v002 delayed a fresh promotee's move to probing, and v003 changed which stash item gets evicted. The Winner is v001, with a small lift on `web_0` 128K and no regressions. v003 lost hit-rate in every cell.

## Patterns

- The stricter room threshold worked as intended. It kept cycle-0003 v002's `web_0` 128K gain and avoided its 16K loss, which confirms that the last-free-slot cascade caused that loss. With 2-slot buckets, though, the bypass fires only when the target is empty, so this lever has little room left.
- `web_0` 128K is still the bottleneck. This cycle ruled out two explanations for it: one-hit churn into probing (v002 made it worse, which suggests hot promotees want to reach probing sooner) and stash flushing of half-gated items (v003). `hm_0` behaves the opposite way and rewards protecting residents. The two traces keep pulling the timing levers in opposite directions.
- Counters that never decay are now a recurring failure mode. v003's "one hit" protection turned into permanent pinning, and v002's target-hit count only weakly filtered anything. Together with cycle 0002's near-zero counters, this means the `lfu_counter` lifecycle (when it resets, whether it decays) shapes the outcome more than any threshold placed on top of it.
- Plain LRU is strongly confirmed as the right stash eviction rule under the two-hit gate. Fresh demotees especially need the recency protection that `Back` demotion gives them.
- Per-op-time and metadata-size are still unreported, for the fourth cycle in a row.

## Open questions

- Is v002's `hm_0` gain worth keeping through a trace-adaptive or occupancy-conditioned version that leaves `web_0` 128K alone? Or is it too small to pursue?
- Does `web_0` 128K benefit from faster target→probing movement, for example promoting on any target hit and not only at the hottest position?
- Would a stash protection that expires (newest region only, or cleared at the LRU end) gain at 16K, or does any departure from LRU hurt?

## Continuation

Build on this cycle's v001. Drop stash victim-selection by counter and stop tuning the free-slot threshold. For `web_0` 128K, probe the opposite direction from v002: let promotees reach probing more easily. Any new frequency signal must come with an explicit decay or reset rule. Measure per-op-time and metadata-size before making cost claims.
