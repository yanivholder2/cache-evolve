# Cross-Cycle Insights

## Recurring wins

- Each cycle's Winner built on the previous one: SLRU (c1), then W-TinyLFU over SLRU (c2), then a tightened sketch (c3), then a capped hill-climbed window (c4). Building on the current champion has worked every time.
- Soft, relative signals beat hard overrides (c1–c4). Examples are SLRU probation, a duel against the current victim with ties admitted, and a gradually adapting window.
- Strict one-shot filtering is what drives hm_0, especially at small capacity. The pieces are the probation segment, the sketch admission gate, dedup on miss-then-put, the doorkeeper, the long reset period and a wide sketch. Together these are the most dependable hm_0 gains (c1–c4).
- Enlarging the front segment (window or HIR queue) is the only lever that consistently lifts small-size web_0 (c3, c4). Adapting the window size with absolute caps on its knobs, plus a floor, fixed this cell without losing anything elsewhere (c4).
- Changing one lever per variant gives clean attribution (c4). Bundled changes in earlier cycles hid the causes.

## Recurring failures

- Hard admission or promotion rules lose on web_0's short-range recency reuse. Examples: TinyLFU over LRU with a strict-greater reject (c1), ghost hit → protected (c2), and an unconditional bypass on in-window reuse (c4).
- Parameters fixed as a share of capacity or scaled linearly with it fail at one end of the size range. Examples: the 1% window and HIR share (c2), the capacity-sized ghost list (c2), and the climber's step and period tied to capacity (c3).
- Loosening frequency memory, through shorter resets or faster decay, gives back hm_0 gains at small size and barely helps web_0 (c3, c4).
- LIRS tuning only trades hm_0 against web_0 and never beat the W-TinyLFU line (c2, c3).
- In all four cycles only hit-rate was measured. `metadata-size` and `per-op-time` were never reported, so every cost claim is unverified.

## Open questions

- Which climber cap does the work, the sample-period cap or the step cap? How far does the window actually move in each trace/size cell?
- With an adaptive window in place, does the doorkeeper still cost anything on web_0?
- Can in-window reuse help as a bounded duel bonus or tie-breaker instead of a bypass?
- How do the sketch, the doorkeeper, the climber's per-access sampling and the ghost/LIRS history compare to plain SLRU on the cost axes? Does CLOCK's expected cost advantage hold?
- Is there one front-segment share that works for both traces, or must it always adapt?

## Per-policy notes

- **LRU (baseline):** every variant since c1 has beaten it.
- **SLRU:** a robust base whose gains grow with capacity (c1). A ghost list feeding protected hurt (c2). If ghost history comes back, it should only feed the admission comparison.
- **CLOCK:** slightly worse on large-size web_0 because its bits stay set and it decays toward FIFO (c1). Kept only for its unmeasured cost advantage.
- **TinyLFU / W-TinyLFU:** plain TinyLFU over LRU failed (c1). With a window in front of SLRU and ties admitted, it has been the lineage of every Winner since c2. Keep the current sketch settings (long reset, doorkeeper, dedup, wide width) and the capped climber with its floor.
- **LIRS:** competitive with LRU but never beat W-TinyLFU. Changing the HIR share is a trade between traces, not a gain. Dropped as of c3.
