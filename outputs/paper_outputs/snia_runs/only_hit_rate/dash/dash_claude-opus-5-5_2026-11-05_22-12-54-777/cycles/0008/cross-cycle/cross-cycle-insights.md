# Cross-Cycle Insights (through cycle 0008)

## Recurring wins

- **One isolated change per variant** gave clean attribution in every cycle. The bundled v001 in c0002 was the only cohort whose regression could not be pinned to a cause.
- **Lineage of Winners:** 24/8 topology (c0001), then the frequency-aware 8-slot tail victim (c0002), then the ghost-credited admission ring (c0004), then a direct-mapped ghost sized to the cache (c0005), then halving on demotion into the stash (c0006), then ghost hits bypassing the stash to target Front (c0007), then a contention-gated bypass (c0008). Each step stacked cleanly on the one before.
- **In-bucket victim choice moves the small cache size; topology does not.** The frequency filter opened small-size gains, and later levers built on it.
- **Admission-side memory (the ghost) is the only family that reached hm_0's long-gap reuse.** Its gains came from three things: reach (c0005), where ageing happens (c0006), and above all **where credited keys land** (c0007). Changing landing was a structural fix that parameter tuning could not have found.
- **Ageing is safe only when applied to items already judged cold**, for example on demotion into the stash. It must not penalise hits or recent Front arrivals.
- **Protect incumbents, not newcomers** (c0008). Fast eviction of un-re-hit ghost arrivals acts as a useful filter. Gating admission on contention helped the small size.
- **Whether a cell responds depends on whether the ghost fires.** hm_0 responds first. web_0 moved only once ghost-driven placement put returners into target at the large size.

## Recurring failures

- **Insert-position changes once the tail scan reads counters.** MRU promotion hurt web_0 in c0002 and again in c0003, and gated demotion did nothing. Front landing is part of how the filter works, because new arrivals must sit inside the scan window.
- **Tuning the tail scan.** Window 8 is the peak: both narrower (c0003) and wider (c0004) lost. Cheaper scan settings could not be justified without cost data.
- **Faster or broader ageing.** Decrement on every eviction failed in c0004 and again in c0007. Removing the stash-hit halving failed in c0005. Every time, the counters of Front-landed items drained before they could protect anything, and small caches were hit hardest.
- **Normal/stash ratio sweeps past 3/4.** The stash is the only path into the protected tier, so shrinking it starves promotion.
- **Ghost bonus size and ghost size above the cache.** Both were exhausted by c0006. Narrow fingerprints cost web_0 through false matches.
- **Direct scan defences for web_0.** A TinyLFU gate on promotion did nothing, because scan keys never reach it.
- **Protecting credited keys inside the victim scan** (c0008). This moves eviction onto proven residents and hurt every cell.
- **Changing where cascade victims go** (stash vs ghost). Hit-rate was identical either way.
- **Measurement gap:** per-op-time and metadata-size have gone unmeasured for 8 cycles. Cost-reduction variants cannot be judged, and the eviction and admission paths keep growing.

## Open questions

- Can a relative bypass gate win back the large-size web_0 gain lost in c0008 while keeping the small-size gains? One option is to bypass when the returner's credit is at least the window's minimum counter.
- Should bypass/stash rules depend on cache size or bucket type, since the trade-off now changes with capacity?
- On a refused bypass, does keeping the ghost entry so the key can retry on a second return help, or does it only delay the loss?
- Is large-size hm_0 structurally flat now? Do its lost keys leave through paths the ghost never records?
- Are there other paths that still send proven-reuse keys through the stash, such as demoted items with prior target hits?
- What do the current Winner's ghost lookup, bypass cascade and 8-slot scan actually cost in per-op-time and metadata-size?
- Gains are shrinking. Is a structurally different hypothesis needed for web_0 at the small size?

## Per-policy notes

These notes cover the single evolving lineage of the segmented normal/stash policy with a frequency-aware tail and a ghost.

- **Topology:** a 24/8 split (about 3/4 normal) is settled. Do not revisit it.
- **Tail victim scan:** keep the 8-slot window, Front landing for arrivals and promotions, and slow halve-on-cap ageing. This setup is load-bearing and its parameters are fully explored.
- **Stash:** halving on a stash hit is the stash's only ageing and must stay. Halving on demotion into the stash is beneficial. If stash ageing changes, move it rather than remove it.
- **Ghost:** direct-mapped, sized to the cache, full-width fingerprints, credit 1. Hits go straight to target Front, gated on whether a cold slot is free.
- **hm_0:** the most responsive trace in both directions. The small-size cell is the most sensitive and shows gains first.
- **web_0:** largely inert. It reacts only through ghost-driven placement at the large size, and it is hurt by false matches and over-crediting.
- **Process:** variant insights should confirm that the overlay diff landed as specified. In c0008 two variants' mechanisms were only inferred.
