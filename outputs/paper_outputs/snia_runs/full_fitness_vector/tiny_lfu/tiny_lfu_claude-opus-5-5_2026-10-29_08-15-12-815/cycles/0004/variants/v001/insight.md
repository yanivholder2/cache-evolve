# Variant v001 Insight

## Helped

- hit-rate: The gain came almost entirely at the 131072 size on both web_0 and hm_0, as the hypothesis predicted. The 16384 size stayed flat on both traces. The parent's increment bumped all four rows on every access, so a key's counters also inflated the estimates of every other key that shared any of those nibbles. Conservative update moves only the rows at the current minimum. That keeps collision noise out of the counters that `estimate_hashed` takes the minimum over. With the large cache, the main region is big and the `cand_freq > victim_freq` contest compares keys with similar, mid-range frequencies. That is where a one- or two-count overestimate decides who wins. At 16384, aging and the 15 cap keep counts low and close together, so less inflation rarely changes the outcome.

## Hurt

- per-op-time (get p50): The median `get` got slower than the parent's, so the "keep get p50 near 35 ns" part of the hypothesis failed. The parent's loop did four independent read-modify-writes. Each had a `c < 15` branch that is almost always taken and so predicts well, and the CPU could overlap all four. The branch-free version builds a serial dependency chain instead: four loads, then three chained `min`s, then four compares against `min`, and only then the stores. Removing a well-predicted branch saved almost nothing. The lost parallelism across rows cost real time.
- per-op-time (get p99, put p50): `get` tail latency and put median got worse for the same reason. Every put and every get runs `increment_hashed` once, so the longer critical path shows up on both operations. Put p99 barely moved, because the eviction's two `estimate_hashed` calls and the map work dominate there.

## Takeaway

Conservative update survives on top of the window/SLRU residency, and its benefit is specific to large caches. The aging cadence is unchanged, because `live` (min < 15) fires exactly when the parent's `moved` flag did. So the hit-rate gain comes purely from less counter inflation. The branch-free form was not free, though. On this path the cost comes from the dependency chain through `min`, not from branch mispredicts. A cheaper version needs to shorten that chain, not just remove the `if`. For example, the min could be computed per word pair with SWAR, or a cheaper approximation could stand in for the exact min.
