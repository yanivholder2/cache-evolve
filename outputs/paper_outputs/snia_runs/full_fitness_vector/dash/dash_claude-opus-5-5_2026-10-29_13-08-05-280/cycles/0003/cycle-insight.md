# Cycle 0003 Insight

## Cohort outcome

All three variants built on cycle 0002's lean threshold-only gate. v001 added an occupancy guard (a third stash hit is needed to enter a *full* target). v002 made the first change to Item layout by narrowing `lfu_counter` to `u8`. v003 gave demoted target residents a re-admission credit. All three are Winners, but only v001 has a mechanistic gain: better large-cache hit-rate and cheaper puts and get tail. v002 left metadata-size unchanged, and v003 lowered hit-rate in every cell. Their latency gains are most likely measurement noise.

## Patterns

- Deciding *who enters* a full target works. Making it *cheaper to come back* does not. v001's guard blocks churn at the door and recovers large-cache hits. v003's credit speeds up swaps between reused residents that share a full target, which adds stash inserts and evicts fresh items that are still waiting for their second hit.
- Stash churn is the shared cost behind both hit-rate and put latency. v001 cut forced demotions into stash and had the clearest put improvement. v003 raised churn and lost hit-rate everywhere. This extends cycle 0001's finding that the number of multi-bucket moves drives both dimensions.
- Protection added by the gate trades large-cache gains against small-cache losses. In small caches, stash residence is too short for a key to collect an extra hit, so a fixed threshold helps one size and hurts the other.
- Metadata-size is set by `timestamp: Instant`, which takes 16 bytes with align 8. The counter is not the constraint. Any field of 8 bytes or less disappears into alignment padding, so narrowing the counter alone cannot shrink Item.
- The per-op-time noise problem is now confirmed in practice. Two variants whose diffs cannot explain their latency moves (put was untouched in both) showed large drops. Latency-only "wins" can't yet be told apart from variance.

## Open questions

- How much noise does per-op-time carry? No variant has re-run an identical baseline yet, and that blocks trust in any latency-only Winner.
- Should the full-target threshold scale with cache or stash size, or apply only when the stash is large enough for three hits to be plausible?
- Can `timestamp` become a ≤4-byte tick, or be dropped for policies that never read it, so that Item fits in 24 bytes without changing hit-rate?
- Does better victim choice inside a full target add anything on top of v001's door guard?

## Continuation

Make v001 (the occupancy guard on full targets) the new baseline. Next, try a size-aware version of its threshold to win back the small-cache slip. Keep v002's `u8` counter as a prerequisite, and attack metadata-size by narrowing or removing `timestamp`. Drop re-admission credits and any other "faster re-promotion" idea. Before accepting another latency-only Winner, re-run the baseline so the size of the measurement noise is known.
