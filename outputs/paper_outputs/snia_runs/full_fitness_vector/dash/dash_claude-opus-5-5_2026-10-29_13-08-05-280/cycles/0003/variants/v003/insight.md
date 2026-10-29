# Variant v003 Insight

## Helped

- per-op-time: The hypothesis predicted no change: the get hit path is identical to v001, and the edit only swaps a `>>= 1` for a constant store on the demotion path. The get median moved a little, which fits that prediction. The large drops in get p99 and in both put percentiles can't be explained by the diff, because `put` was not edited and the extra work happens only on demotion. Treat them as measurement variance until a re-run of v001 and v003 on the same host confirms them. They are not evidence that the re-admission credit makes anything faster.

## Hurt

- hit-rate: Hit-rate went down slightly in all four cells, including the large hm_0 cell the change was aimed at. A stash hit already counts as a cache hit, so moving a credited item back into target earns no extra hits. Each one-hit re-promotion goes into a full target with no victim check, so it forces the next LRU resident into stash, and that resident also gets the credit. Two reused keys that share a target can then swap places on every access. Each swap is another stash insert, which pushes out fresh items that are still waiting for their second hit.

## Takeaway

v001's large-cache loss did not come from the counter a displaced resident has when it lands in stash. Making re-promotion cheaper only speeds up the swapping between reused residents of a full target: hit-rate does not improve, and stash churn goes up. To win back those hits, the variant has to decide who gets evicted from target (keep the displaced resident, or choose a better victim) or how long items survive in stash. Changing how quickly an item leaves stash does not help.
