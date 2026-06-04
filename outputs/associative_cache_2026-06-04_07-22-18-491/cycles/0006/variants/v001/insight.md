# Variant v001 Insight

## Helped

- hit-rate: Adding the stash gave primary evictions a recovery path — items that were warm but had accumulated low `lfu_counter` relative to their bucket peers were no longer permanently discarded, and the LFU-weighted `eviction_candidate_index` in `ClassicLRU` ensured that the true lowest-frequency resident was chosen as the victim rather than whichever item happened to be oldest in position.

## Hurt

## Takeaway

Layering the stash and LFU-weighted eviction on top of the existing probe zone produced a meaningful hit-rate gain because they target a failure mode the probe zone cannot address: items that passed the probe admission filter and entered primary but were later evicted under a cold burst before accumulating enough frequency to survive. The stash intercepts those evictions and gives them a second chance on re-access, while LFU eviction selects better victims from the start. The two mechanisms reinforce each other — better victim selection reduces stash churn, and the stash limits the cost of the occasional wrong eviction choice.
