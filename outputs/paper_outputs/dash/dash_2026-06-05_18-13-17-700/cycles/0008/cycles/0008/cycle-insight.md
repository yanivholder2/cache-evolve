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
