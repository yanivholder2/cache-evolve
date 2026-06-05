# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle ran two converging tests on the promotion gate and opened one
brand-new seam on the stash. The promotion-side probes — a density-independent
per-key touch counter (v001) and loosening the len-2 hottest-only boundary
(v003) — came back flat and slightly negative respectively. The genuinely new
lever, a cuckoo-style stash relocation hop from a clean baseline (v002), was
the only clean positive move. The winner was v001, but its gain was
noise-level; v002 is the more meaningful result.

## Patterns

- The promotion gate is now closed from two independent directions. The touch
  counter replaced *rank* with a density-independent warmth signal and still
  produced nothing — proving the small-bucket ceiling is structural, not a
  rank artifact. The len-2 loosening over-admitted and edged down — proving the
  withheld second resident is genuinely cold, not a warm key held back. Both
  reads point the same way: no promotion-side signal, of any shape, beats the
  occupancy ceiling on this trace.
- The stash's load-bearing property remains *dwell time*, and it is the only
  axis still yielding gains. A single relocation hop into a salt-derived
  alternate bucket lets a few more cold keys catch their second reference
  before dropping — positive, no regression, capacity conserved. This is the
  freshest live lever in the whole search.
- The relocation gain is marginal for a structural reason, not a logic flaw:
  the alternate stash bucket is usually full too, so one hop rescues only a
  sliver of the dwell-time headroom the stash holds.

## Open questions

- The stash relocation is the first new positive seam in several cycles — does
  it deepen (multi-hop cascade, smarter overflow-victim selection, larger
  alternate set) or does it saturate as fast as every prior lever once the
  alternate buckets fill?
- With the promotion gate now exhausted by both rank and touch-count signals,
  is *every* hit-rate lever on this trace ultimately gated by the same hashing
  skew, or is the stash dwell-time axis structurally different?

## Continuation

Stop all promotion-gate work — rank, fraction, and now density-independent
touch-counting are all confirmed dead against the small-bucket ceiling, and
the len-2 boundary is doing real work that must not be loosened. The live
frontier is stash dwell-time: v002's relocation hop is the only fresh positive
result, so next cycle should deepen it (better victim selection, a second hop,
or a larger alternate set) rather than chase more promotion variants. The
recurring skew/occupancy bottleneck — now blocking levers across five cycles —
makes validating against a second, less-skewed trace overdue before investing
further.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
