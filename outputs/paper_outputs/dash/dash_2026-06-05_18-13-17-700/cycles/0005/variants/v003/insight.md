# Variant v003 Insight

## Helped

- hit-rate: Splitting promotion into two speeds — landing the stash→target hop at the LRU end (`InsertPosition::Front`) while keeping target→probing at the MRU end (`InsertPosition::Back`) — makes a freshly-admitted key climb back to the freshest quarter before it can reach probing, so one-hit cold spikes no longer rocket into the protected tier and displace the genuine hot set, yielding a small gain over the parent's uniform-Back promotion.

## Hurt

- hit-rate: The gain is only a sliver because the slow first hop interacts with the inherited `is_in_top_quarter` gate's `len < 4` fallback — on this heavily-colliding trace target buckets are usually small, so a Front-landed key climbs to the lone hottest slot in just an access or two anyway, leaving little daylight between "re-proved warmth" and the parent's single-hop behavior.

## Takeaway

Two-speed promotion is a genuinely distinct lever from gate-widening and the global Back flip: reserving probing for keys with two demonstrated accesses guards the top tier against one-hit cold spikes and does nudge hit-rate up. But on a trace where target buckets rarely hold four-plus residents, an LRU-landed key reaches promotable rank almost immediately, so the "must re-prove" barrier is shallow and the lever delivers only a fraction of its intended effect — the same bucket-occupancy bottleneck that capped the parent's gate widening.
