# Cycle 0010 Insight

## Cohort outcome

Variants tried sampled-counter adaptive insertion depth (v001), a three-way rotation that cascades the displaced len/4 entry through len/2 (v002), and a 4-tag ghost ring of recent evictions that deepens insertion for re-referenced keys (v003). v003 and v002 were Winners, but only v003 shows a real gain: a modest hit-rate improvement at a put-time cost.

## Patterns

- The Winner-versus-evidence mismatch continues, an eighth straight cycle. v002's own insight reports lower hit rate and slower get and put, yet it won.
- A real reuse signal beat proxies. The ghost ring tracks actual re-references of evicted keys and is the first idea in several cycles to improve hit rate cleanly. Sampled global hit ratio (v001) and position-based demotion (v002) did not, which agrees with cycle 0009's rejection of weak proxies.
- Displacement handling is not the lever. Moving the swapped-out entry to len/2 instead of the tail only moved the cost, so the issue is eviction choice, not where the displaced entry lands.
- Put-path work is not free. The ghost ring, the sampled counters and the extra moves all raised put time, contradicting the assumption that put-miss additions are cheap.
- Get latency rose in all three even where the get path was untouched, which points to layout or noise. Get still has no real gain, now eight cycles.
- No variant grew reported metadata.

## Open questions

- Is the get-latency rise in v003 real or a layout or noise effect?
- Can the ghost ring be made cheaper, for example with fewer compares, a smaller tag or a hash filter?
- Does a larger ring, or using ghost hits to also steer eviction, add hit rate?
- Do results hold on a second workload, and are Winner statuses beyond noise?

## Continuation

Make the ghost-ring reuse signal from v003 the lead and work on reducing its put cost and extending its benefit, such as ring size or ghost-guided eviction. Drop sampled global-ratio adaptivity and cascaded or longer displacement chains. Add a noise check on get latency and a second workload, and keep discounting Winners whose own insight shows a loss.
