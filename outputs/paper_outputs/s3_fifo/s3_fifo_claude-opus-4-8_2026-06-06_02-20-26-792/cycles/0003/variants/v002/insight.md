# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Routing stale (back-half) ghost hits into `small` with `freq = 0` instead of straight to `main` demoted genuine long-gap reuse back behind the admission filter, where one more pass without a re-hit drops it to ghost again — so the freshness gate suppressed exactly the reuse the deeper ghost was meant to capture, edging hit-rate below the parent.

## Takeaway

The untried middle path — deeper ghost (`2 * main_size`) plus a freshness gate — does not recover what cycle 0001 and 0002 lost; splitting ghost hits by queue position treats a confirmed reuse signal as if it were weaker, and re-admitting stale hits via `small` discards that signal almost as fast as the deeper window earns it. On this workload, every ghost hit is strong evidence and belongs direct-to-main regardless of recency; position-based gating is the wrong axis for re-admission here.
