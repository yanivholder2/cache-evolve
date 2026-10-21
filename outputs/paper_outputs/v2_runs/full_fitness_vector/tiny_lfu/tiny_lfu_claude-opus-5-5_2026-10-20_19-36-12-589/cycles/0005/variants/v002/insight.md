# Variant v002 Insight

## Helped

- hit-rate: Skipping the second `increment_hash` in `put` after a `get` miss on the same key means each access now adds +1 whether it was a hit or a miss. The `incoming_freq > victim_freq` duel therefore stops giving one-hit newcomers a free extra count, and a few more cold keys are kept out. The gain is small, and the double count explains why. The bias was never one-sided: every resident, the LRU-head victim included, also got in through a double-counted miss+put. Rejected keys that come back also got +2 on every re-miss. So the bias mostly cancelled in the duel, and fixing it only matters in the narrow cases where newcomer and victim estimates were within about one count.

## Hurt

- per-op-time: `get` and `put` both got slower at p50 and p99. `put` p99 regressed the most. The code adds very little: two field stores in `get` (`last_miss_hash`/`last_miss_valid`), now split across a `match` on hit vs miss, plus one compare-and-branch in `put` that picks `estimate_hash` or `increment_hash`. On the miss→put path, `put` swaps a read-modify-write for a read, so it does less work. None of that explains a slowdown this large. The cache struct did grow by 16 bytes (u64 + bool + padding), but the hot path doesn't touch those fields in a way that should cost tens of ns. The likeliest explanations are bench drift, which the parent insight already flagged, or a codegen change from the new branch in `put` (for example, `increment_hash` no longer being inlined into one straight-line path). Re-measure before treating this as a real cost of the fix.

## Takeaway

Removing the get-miss/put double count is correct, since the new `miss_then_put_counts_once` test pins the sketch to one count per access. It costs no per-entry memory, but it moves hit-rate only a little because the bias inflated newcomers and residents almost equally. The +1 vs +2 accounting asymmetry was not what was letting one-hit wonders win duels on workload-a. Together with the parent's finding that sketch accuracy is nearly used up, this points to the victim-selection and aging policy as the remaining lever, not frequency bookkeeping.
