# Cycle 0008 Insight

## Cohort outcome

All three variants forked from the cycle-0007 incumbents and attempted
"behavior-preserving" structural moves on the dash segment: reorder the get
lookup branches stash-last (v001), relocate the per-put heat clear to the
demotion site (v002), and collapse the two-tier demotion cascade to a single
stash hop (v003). All three entered the Winners, but none was a clean win —
each move's premise was contradicted by where the work actually lives, and the
cohort overturned two standing assumptions at once.

## Patterns

- The get median lives in the **stash**, not the promoted tiers. v001 assumed
  the hottest hits resolve in target/probing and pay a wasted stash scan, so it
  moved stash last — and get-p50 regressed, because the median get hit actually
  resolves in the stash and now pays two failed scans before reaching it. The
  branch-ordering lever is real but points stash-*first*, the inverse of the
  hypothesis. Behavior was preserved exactly (hit-rate/metadata bit-for-bit);
  only the cost model was inverted.
- Relocating a probe off the median path trades median for tails. v002 did
  shave put-p50 by deleting the unconditional per-put heat clear, confirming
  that touch was real hot-path work — but the clear didn't vanish, it landed on
  the demotion branches and resurfaced as wider put-p99/get-p99 and a worse
  get-p50 summary. Moving work off the median is not free when the destination
  adds branching the tail then exposes — the same lesson cycle 0007's v003 taught
  on the eviction path.
- Hit-rate is **not** perfectly conflict-insensitive. v003 cut the put-p99
  cascade exactly as cycle 0007 predicted by collapsing the foreign-resident
  retry to a single stash hop — but hit-rate slipped below the parent, because
  the one-more-chance re-home was salvaging a small number of hits. This is the
  first crack in the multi-cycle "hit-rate fully capped / conflict-insensitive"
  framing: the demotion cascade is a genuine latency-vs-hit-rate trade, not pure
  overhead.

## Open questions

- If the get median resolves in the stash, is a stash-first reordering an
  actual get-p50 win, or does promoting the stash branch only relocate the cost
  onto the promoted-tier hits the way v002 relocated the put clear?
- The put-p99 cascade is now a priced trade: how large is the hit-rate cost of
  the single-hop demotion, and is there a cheaper conflict-mitigation step that
  keeps most of the rescued hits without the double-hop tail?

## Continuation

Two assumptions this cohort relied on were wrong and should be dropped: hot
get hits do **not** live in the promoted tiers (the median is in the stash, so
try stash-*first* if probing get-p50 at all), and the demotion cascade is **not**
free on hit-rate (collapsing it trades a few hits for the put-p99 win). The
put-p99/hit-rate trade (v003) and the put-p50/tail trade (v002) are both now
characterized as trades, not clean wins — next cycle should decide whether
either trade is worth banking rather than seeking a strictly-free version, and
should treat "hit-rate fully capped" as no longer exact in the conflict/demotion
machinery.
