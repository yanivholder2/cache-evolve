## Recurring wins

- Isolating one change per variant has kept attribution clean. The one bundled variant (c0002 v001) left its web_0 loss impossible to pin to a single cause until c0003 re-tested its parts separately.
- Frequency-aware victim selection on the normal-tier tail (8-slot window) is the main hit-rate engine. It added cleanly on top of the 24/8 topology and was the first lever to move the small cache size.
- Admission-side memory (ghost credit) is the only family that has reached hm_0's long-gap reuse. A direct-mapped ghost sized to the cache beat a small ring, and it also turned ghost lookup into an O(1) operation.
- Counter ageing at the right point keeps paying off (c0004, c0005, c0006). It works when it is placed where no hit is penalised (halving on demotion into the stash). The useful effect is changing who ghost-credited arrivals compete against.
- Gains concentrate in small-size hm_0, where frequency matters more than recency. That cell is the earliest and most sensitive signal for any filter or admission change.

## Recurring failures

- Insert-position changes. MRU landing on promotion hurt web_0 in c0002 (bundled) and again in c0003 (alone). Front landing is part of how the tail filter works, and gated demotion turned out to be redundant with the scan.
- Topology tuning past 3/4 normal. Shrinking the stash starved the only route into the protected tier, and topology alone never helped the small size.
- Tuning the tail-scan parameters. Window 8 is mapped from both sides (narrower loses the filter, wider becomes coarse LFU). Faster ageing drains counts before they can protect anything.
- Removing ageing anywhere. Dropping the stash-hit halving hurt every cell, because high counters plus Front landing make items stick in target.
- Scaling the ghost further. Bonus strength (remembered+2, credit 2) and reach beyond 1× capacity are both used up; 2× dilutes the bonus. Narrowing the fingerprint costs web_0 through false matches.
- Per-op-time and metadata-size have gone unmeasured for six cycles despite repeated requests. Every cost argument (scan window, ghost tables, ring vs O(1)) is still unverified.

## Open questions

- Does ageing at other tier boundaries help the way demotion-ageing did? Candidates: halving on stash→target promotion, or ageing inside the normal-tier scan.
- Does large-size hm_0 lose reuse through exit paths the ghost never records?
- Would a stronger demotion decay (a quarter, or a reset) help hm_0 further, or start hurting web_0 at the large size?
- Is there any lever for web_0? It has been inert or slightly negative to every change so far. Untried candidates: scan detection, or admission filtering at the normal tier.
- Gains have shrunk every cycle since c0004. Is the current structure near its limit, so that a structurally different hypothesis is needed?
- Would window or ageing settings scaled by cache size or bucket type capture small-hm_0 gains without the losses elsewhere?

## Per-policy notes

- Baseline (target/probing protected tier, normal/stash buckets): its protected tier gave almost no protection. The c0001 fixes all addressed this from different sides.
- Topology: 24 normal / 8 stash is the settled split. With admit-everything-to-stash, the stash is the gateway to promotion, not just a scan filter.
- Tail-victim filter: keep window 8 and slow, rare ageing (halve-on-cap). Its parameters are spent as levers.
- Ghost admission: lineage is ring (c0004) → direct-mapped table sized to the cache, credit 1, full fingerprints (c0005, current form). Ghost-size and ghost-bonus tuning is ruled out.
- Current parent: c0006 v003, which halves an item's counter on demotion into the stash, on top of the capacity-sized ghost.
- Traces: hm_0 is reuse-heavy and responds in both directions. web_0 is scan-heavy and has barely moved under any lever.
