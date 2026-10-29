# Variant v003 Insight

## Helped

- hit-rate (16K, `web_0`): A small gain over the parent. This fits the hypothesis's stale-credit argument. A shorter ring stops crediting keys that return after more than one stash-capacity of evictions. Those late returners were taking a fast-track promotion into a target bucket that a fresher resident deserved.
- hit-rate (128K, `web_0`): Also a small gain, not the loss one might expect. The ghost-credited returns that drive this cell seem to come back within one stash-capacity of their eviction. Halving the ring kept them and dropped only the tail credit, which was slightly harmful.
- hit-rate (16K, `hm_0`): Flat. The parent's insight already noted that the ring's reach is short relative to reuse distance at 16K, so halving it changes little here.

## Hurt

- hit-rate (128K, `hm_0`): The one cell that slipped. At the large size, `hm_0` has keys that return between one and two stash-capacities after eviction. Factor 2 credited them and factor 1 forgets them, so they need two stash hits again before promotion. This cell had the parent's biggest combined ghost + MRU-probing gain, so it is the most exposed to lost ghost reach.

## Takeaway

Halving the ghost ring is close to hit-rate-neutral on aggregate, which met the hypothesis's "at or above parent" bar only at the margin. So most of the ghost's value comes from returns within one stash-capacity. Reach still has a trace-dependent optimum: `web_0` gains slightly from less stale credit, while `hm_0` 128K still wants the longer tail. Factor 1 is the better cost/benefit point only if the halved ghost metadata and shorter `GhostRing::remove` scan actually show up. Those dimensions were not measured in this eval, so the claimed cost savings are still unverified.
