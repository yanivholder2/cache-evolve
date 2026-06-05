# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle drilled entirely into cycle 0006's one fresh seam — the stash
relocation hop — testing three ways to deepen it: grafting the single hop
onto the proven protected-tier winner (v001), balancing admission across both
homes via power-of-two-choices (v002), and extending the single hop into a
bounded multi-hop cuckoo cascade (v003). The declared winner was v001, but its
gain was noise-level; v003's cascade was the only clean, regression-free
positive and is the meaningful result.

## Patterns

- The stash dwell-time seam responds to *depth*, not breadth. A bounded
  multi-hop cascade — evicting an LRU resident at each full bucket and chasing
  free space a few buckets deep — converts would-be drops into survivors and
  lifts hit-rate with capacity conserved. The single hop left this headroom on
  the table precisely because the lone alternate home was usually full too.
- Pre-emptive load-balancing is the wrong frame for the same seam. Spreading
  cold keys evenly across both homes at admission time scatters keys away from
  the primary-first locality the get path expects, deferring rather than
  preventing drops, and regressed. Reactive rescue of genuine collision victims
  beats proactive evenness — the value is dwell time, not balanced occupancy.
- The late slivers do not compound. Stacking the single-hop relocation onto the
  protected-tier/two-speed-promotion winner came back flat: the two seams chase
  the same boundary key pool, and keys the relocation would rescue are already
  reclaimed through promotion. Independently-discovered gains overlap rather
  than add.

## Open questions

- Does the cascade keep paying out if deepened further (larger hop budget,
  smarter victim selection than LRU), or does it saturate once the reachable
  bucket neighborhood fills — the same occupancy ceiling in a new guise?
- With combination shown non-additive here, is there *any* pair of discovered
  seams on this trace that stacks, or is every lever ultimately competing for
  the same thin pool of boundary-oscillating keys?

## Continuation

The live frontier is the stash cascade depth (v003), not the single hop or
balanced admission — drop power-of-two-choices admission (it regressed; primary
locality matters more than even fill) and stop trying to graft the single hop
onto the protected-tier winner (confirmed overlapping, not additive). Next
cycle should deepen v003: test victim-selection policy and hop-budget
sensitivity to find where the cascade saturates. The recurring finding that
seams overlap rather than compound, plus the five-cycle skew/occupancy
bottleneck, makes validating against a second, less-skewed trace overdue before
investing further in this single workload.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle stress-tested whether cycle 0007's stash-relocation seam still
deepens once stacked on the protected-tier winner, attacking the same
"alternate is usually full too" cause from three angles: cascade *depth* ported
onto the winner (v001), occupancy-aware *victim quality* inside the cascade
(v002), and *breadth* via a third salted home (v003). All three came back flat
or negative; the declared winner (v003) was a noise-level tie, not a real gain.

## Patterns

- The stash-relocation seam is now closed in all three of its remaining
  directions on top of the protected-tier winner. Depth regressed, victim
  quality was inert, and breadth was exactly flat — three independent reads
  converging on the same exhaustion verdict.
- Depth *inverts* once the protected tier is present. On a clean baseline the
  cascade beat the single hop, but here it evicts established stash residents —
  which two-speed promotion has already warmed — to rescue a colder overflow
  victim, so the rescue costs more than it returns. The clean-baseline win did
  not survive contact with the winner.
- Victim quality and breadth are not independent levers from depth. Full
  alternates are rare enough, and a displaced resident's own clean home
  reachable in few enough cases, that choosing a tidier victim or adding a
  third home yields the same survivor set. This re-confirms cycle 0007's
  finding: independently-discovered seams overlap rather than compound, because
  the keys they rescue are already reclaimed through promotion.

## Open questions

- With depth, victim quality, and breadth all exhausted against the
  protected-tier winner, is there *any* untried shape of the stash seam left, or
  is dwell-time fully harvested on this trace?
- Every lever for four-plus cycles now overlaps the protected tier rather than
  stacking — is non-additivity a property of this single skewed workload, or a
  general truth that only a second trace can disambiguate?

## Continuation

Stop deepening the stash-relocation seam: depth (regressed), victim selection
(flat), and breadth-of-homes (flat) are now all confirmed dead on top of the
protected-tier winner, which remains the best-so-far unchanged. The repeated
finding that every fresh seam overlaps the protected tier rather than
compounding — now across multiple cycles — means in-trace search has very
likely saturated. The long-deferred validation against a second, less-skewed
trace is now the highest-value next step; further single-workload variants are
unlikely to bank more than slivers.


### Cycle 0009

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
