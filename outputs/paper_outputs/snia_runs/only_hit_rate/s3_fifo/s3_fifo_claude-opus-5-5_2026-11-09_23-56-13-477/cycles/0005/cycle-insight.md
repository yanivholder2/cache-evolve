# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle 0004's v001 (freq-gated ghost path). v001 added the one-time small relap on top of it. v002 split freq-0 ghost returnees by their own ghost age, so quick returns go straight to main. v003 ported the cost-only changes and left hit-rate bit-identical. v002 won. It is the first variant in this lineage to gain on both traces at the small size without losing in any cell.

## Patterns

- Per-key evidence keeps paying off, and how recently a key returns is a stronger signal than whether it was reused in small. A key's ghost age costs nothing to read, since stamps already count ghost insertions, and it separated hm_0's real reuses (squeezed out of a small queue) from web_0's late, one-off rereferences. Global rate proxies never managed that split.
- Both policy variants depend on the "reused once" signal the ghost gate uses. v001's relap resets freq to 0, which erases that signal, so failed relaps lost their direct ghost→main return. That probably cancelled the gain on hm_0 at the small size. Mechanisms layered onto the ghost gate have to preserve the per-key evidence it reads.
- Windows that scale with `small_size` reach the large cache size. Before this cycle, policy gains were limited to the small size. Here v001's relap and v002's age window both moved cells at the large size, because their windows grow with the small queue. web_0 at the large size remains the one cell neither variant moved.
- Per-op-time and metadata-size are still not in the fitness vector, for the fifth cycle running. This holds even though H03 assumed the leaderboard now reports them. Three cost-only variants are now waiting on measurement, and v003 is the one that combines the cost changes with the winning policy line.

## Open questions

- Does widening or narrowing the age window (`GHOST_RECENT_AGE_DIVISOR`) move web_0 at the large size?
- Does v001's relap add to v002's gain if a failed relap is ghosted with freq 1 (or marked as relapped), so it keeps its ghost→main return?
- How much do the cost changes actually save once the cost dimensions are measured? On 8-byte keys, alignment may cancel the ghost-deque saving.

## Continuation

Use v002 (the age-gated ghost path) as the policy parent, and port v003's cost changes onto it, since they leave hit/miss behaviour identical. Next, sweep the age window, and retry the relap on top of v002 with failed relaps keeping their freq-1 ghost signal. Drop freq-resetting relaps. Do not spend further hypotheses on cost work until per-op-time and metadata-size actually appear in the fitness vector.
