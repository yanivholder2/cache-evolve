## Recurring wins

- Eviction *ordering*, not extra capacity, is where the gains live: every
  win held capacity fixed and improved which resident key dies first.
- "Protect on second access" is the durable axis — distinguishing
  one-hit-wonders from genuine reuse, encoded as segmentation, beat pure
  recency and seeded every subsequent winner.
- Enlarging the protected tier keeps paying: each cycle that pushed
  protected residency higher (segmentation, then 80/20, then 90/10) won,
  confirming protected-tier room as the dominant lever on this trace.

## Recurring failures

- Loosening the promotion path consistently backfires — delaying
  promotion to a second hit, widening the probationary segment, and
  over-trusting a single post-eviction reuse all lost hits. Reuse arrives
  fast and within a narrow probationary window; first-hit promotion is
  confirmed correct.
- Cleverer eviction-order machinery (frequency-windowed victim selection,
  ghost / reuse-distance memory) is near-inert as a standalone lever:
  second-access promotion already captures most reuse before eviction
  order matters, leaving only a thin warm-but-unpromoted band to rescue.

## Open questions

- Where is the protected-ratio ceiling? Probing past 90/10 (e.g. 95/5)
  must eventually break the eviction path once probationary nears empty.
- Do the confirmed-marginal levers (frequency window, weak-protection
  ghost) contribute anything when *layered on* the maximally-protected
  winner rather than tested in isolation on the original parent?
- With promotion and ratio both near-tuned, is the trace's recency
  structure largely consumed — is there an untapped, non-recency workload
  signal worth a fundamentally different lever?

## Per-policy notes

- SLRU (segmented): the through-line winner. Strong local optimum whose
  bias is "promote early, protect generously"; gains come from shifting
  the protected/probationary split toward protected.
- Single-list approximations (midpoint insertion, CLOCK reference bit):
  helped over pure LRU as cheap "second access earns protection" signals,
  but explicit segmentation outscored them.
- Ghost / reuse-distance lists: directionally right only when routed to
  the protected-LRU end (weak protection); promoting ghost hits straight
  to protected-MRU crowds the hot set and regresses. Recoverable headroom
  is small either way.
- Pure-recency LRU baselines: dominated on this reuse-heavy trace; retired
  as probes.
