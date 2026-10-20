# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Capping counters at 15 and halving every 10×capacity effective increments squeezes every frequency estimate into a narrow band (0–15, cut back to 0–7 at each reset). Under the unchanged strict `incoming_freq > victim_freq` admission rule, many more contests now end in ties. The LRU-head victim always wins a tie, so this variant still has no window or doorkeeper. Also, the strong long-term frequency gaps that v003's never-aging counters relied on on workload-a get cut down to tie-prone values. The sketch now ranks keys less well, and the hit rate dropped instead of rising.
- per-op-time: Both get and put pay this cost, because both call `increment_hash`. The saturation check `*counter < MAX_COUNT` used to be always true and perfectly predicted. Now it depends on the data and flips often, since hot keys sit at the cap. On top of that, the new `added` flag, the `additions` counter and the threshold check sit in the hot path. Every sample window, `reset()` also sweeps the whole flat `counts` table, which is O(depth×width), and that sweep pushes other data out of the CPU cache. Together these show up at both p50 and p99, well beyond the "near v003" that the hypothesis predicted.
- metadata-size (secondary entry): `TinyLfuCache<u64,u64>` grew by the two new `usize` fields (`additions`, `sample_size`) inside the inline `CountMinSketch`. The headline `Node` size and the sketch's heap size are unchanged, as the hypothesis required.

## Takeaway

Caffeine-style aging and a 4-bit cap on their own, with the strict-greater admission rule kept and no admission window, made v003's TinyLfu worse on workload-a. The aging made counts fresher but less precise, and in a design where ties keep the victim, losing precision means more good newcomers are rejected. The aging also wasn't free: the saturation and reset bookkeeping inside `increment_hash` about doubled per-op latency. Before aging can pay off, it probably needs a companion change to how ties or admission are handled, and a cheaper way to do the reset bookkeeping.
