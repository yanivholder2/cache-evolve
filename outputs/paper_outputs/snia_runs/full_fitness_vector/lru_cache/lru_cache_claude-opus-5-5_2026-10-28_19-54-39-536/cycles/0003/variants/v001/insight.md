# Variant v001 Insight

## Helped

- hit-rate (hm_0, both sizes): Here the ghost does what the hypothesis intended. A hm_0 key that falls off the 20% probation segment before its second reference is recognised when it comes back and goes straight to protected. The gain is largest at the small size, where probation is too short to catch medium reuse distances.
- metadata-size: `Node<u64,u64>` stays at 24 bytes because the ghost lives in per-cache `HashSet`/`VecDeque` fields, not in the node. The summary dimension is unchanged, as step 1 of the hypothesis required.

## Hurt

- hit-rate (web_0, both sizes): The prediction was backwards. web_0 at the large size, which was meant to gain the most, lost the most. Every ghost hit is inserted at protected MRU and calls `demote_if_over`, which pushes the protected LRU out to probation. With `ghost_cap = capacity/2`, web_0's long-distance returners keep displacing protected entries that were proven hot by a real resident hit. The result is a net loss, and the gains on hm_0 only just offset it in the summary.
- per-op-time (put): Every miss now pays for a `ghost_set.remove(&key)` lookup. Every probation eviction also pays for a key clone, a `HashSet` insert, a `VecDeque` push, and an overflow pop with a second `HashSet` remove. That adds about three hash operations to a miss path that already does a map remove and insert.
- per-op-time (get): `get` p50 and p99 rose even though the code path is unchanged. The likely cause is memory, not instructions. A ghost `HashSet` of up to half the capacity, plus its FIFO, competes with `map` and `nodes` for CPU cache, so the same hash lookup and relink in `get` miss cache more often. The summary metric is get p50, so this is the costliest effect of the change.
- metadata-size (secondary, `LruCache` struct): The header grew to hold the `HashSet`, the `VecDeque` and `ghost_cap`. This is a fixed cost per cache and does not count toward the summary, but it adds to the secondary row.

## Takeaway

An A1out ghost that sends a returning key directly to protected MRU is too aggressive for web_0. On that trace a second reference at long distance is weaker evidence than a resident hit, yet this variant ranks the two equally and demotes protected entries to make room. The ghost also costs far more than "no hit-path cost": a separate hashed structure of half the capacity slows `get` through cache pressure even when `get` never touches it. The summary hit-rate barely moved. A ghost is worth keeping only if it is much smaller or cheaper, for example fingerprints instead of full keys or a smaller `ghost_cap`. It also needs a gentler admission rule, such as probation MRU or protected without forced demotion.
