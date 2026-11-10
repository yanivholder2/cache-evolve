# Variant v001 Insight

## Helped

- hit-rate: The only gain is web_0 at 16384. There, main is small and turns over fast, so letting a one-shot returner into main on a single ghost hit costs a live resident. The extra probation pass in small filters those returners out, which is the mechanism the hypothesis described. It just shows up at the opposite cache size from the one the hypothesis targeted.

## Hurt

- hit-rate: web_0 at 131072 went down, the opposite of what H01 claimed, and hm_0 went down at both sizes, so the summary fell. A small-demotee ghost hit identifies a key whose reuse distance is longer than one pass through small but still inside the ghost window. Sending that key back into the same 1/4-capacity small queue and asking for another hit within one pass tests it against the very window it already failed. Many such keys cycle ghost → small → ghost and never reach main. Meanwhile they take small slots away from new arrivals.
- per-op-time: The `put` tail grew sharply, and `put` p50 grew noticeably. Before, a ghost hit cost at most one `evict_main()`. Now a small-demotee ghost hit runs `evict_small()`, which can either graduate an entry (and cascade into `evict_main()`) or demote one (`push_ghost`). Re-admitted keys that fail probation are ghosted again, so ghost churn goes up. That makes the O(ghost) `retain` compaction in `push_ghost` run more often. The `get` change is a single ns and is noise, since `get()` is unchanged.

(metadata-size did not move. The source bit lives inside the existing `u64` ghost tag, and `Entry<u64,u64>` stays 24 bytes.)

## Takeaway

On this parent, a small-demotee ghost hit is already the "confirming hit" for keys whose reuse gap is longer than small. Making them earn a second hit inside small mostly throws away the recency signal the ghost exists to capture. The hurt is largest on hm_0 and on web_0 at large sizes, where main has room for them. Stricter ghost-to-main admission joins looser admission (cycle-6 v003) as a loss for web_0 at 131072. That points away from the ghost handoff rule as the lever for that gap. Any future probation scheme for ghost returners would also need to avoid the extra `evict_small`/ghost-churn work it adds to `put`.
