# Cycle 0009 Insight

## Cohort outcome

After several cycles of the stash-relocation seam saturating, this cycle
broke out of it by aiming at *populations the stash seam never reached*:
a ghost list fast-tracking dropped-then-recurring keys (v002), a
breadth-of-homes hop on the warm probing tier (v003), and a head-on
width-for-addressability trade on the bucket geometry (v001). Both
new-population probes landed small clean positives with no regression;
v002 won, and the geometry trade was a flat wash.

## Patterns

- *Which* keys a lever spares matters more than the lever's shape. The
  identical breadth-of-homes idea that was inert on cold stash overflow
  (cycle 0008) paid off when applied to the probing tier (v003), because
  probing victims are proven-warm residents that earned promotion, not
  cold overflow already reclaimed elsewhere. Same mechanism, hotter pool,
  real gain.
- Orthogonality, not depth, is what finally stacked on the winner. v002's
  ghost gate targets keys that were fully evicted then re-requested — a
  LIRS-style short-reuse-distance population that promotion never
  recaptures — so it nets positive instead of washing out like every prior
  stash variant that chased the same warm pool the protected tier held.
- The occupancy ceiling is set by total capacity, not its partition.
  Halving bucket width while doubling count (v001) was exactly neutral:
  under a conserved budget, addressability and depth are interchangeable —
  every slot narrower buckets remove from depth is handed back as a new
  bucket, so the same hot key overflows either way. The predicted
  "addressability beats depth" lever is neutral at this geometry.

## Open questions

- Both new positives are small because the populations they serve are thin
  on this workload (the dropped-then-recurring set; the narrow slot-legal
  probing-relocation case). Do these orthogonal seams compound with each
  other, or do they too overlap once both ride the winner?
- The "spare the warm victims, not the cold ones" principle reframes
  several previously-dead levers — are there other tier-mirror reapplications
  of inert stash ideas onto the warm probing/protected tiers worth testing?

## Continuation

Stop trying to break the per-bucket occupancy ceiling via geometry
partition (width/count trade is confirmed neutral under conserved budget).
The live frontier is *orthogonal, warm-population* seams: v002's ghost-gate
on dropped-then-recurring keys is the new best and v003's probing-tier
breadth is independently positive — next cycle should test whether these
two stack together, and apply the "spare warm victims" principle to other
inert stash ideas re-pointed at the probing/protected tiers. The
long-deferred second-trace validation remains overdue: these gains are
narrow because their populations are thin in this single workload.
