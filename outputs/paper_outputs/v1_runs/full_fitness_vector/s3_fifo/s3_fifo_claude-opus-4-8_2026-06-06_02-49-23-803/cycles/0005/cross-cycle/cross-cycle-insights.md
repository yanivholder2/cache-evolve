# Cross-Cycle Insights

## Recurring wins

- **Capacity reallocation (small/main split):** moving room out of the
  admission filter into main lifted hit-rate every cycle it was tried
  (0001–0003). Monotone but steadily flattening; the small queue only
  needs to be wide enough to screen one-hit-wonders.
- **Rewarding demonstrated reuse (freq-ceiling):** deepening durability
  for already-hot keys is a small, real, and *orthogonal* gain — it never
  loosens the strict filter, so it stacks cleanly with the split.
- **Ghost length at ~50% of main:** the standout result — a non-tradeoff
  point that improved put-latency *and* hit-rate at once. The reuse this
  trace rewards lives in the 25–50% band of recent evictions.
- **Metadata discipline:** every winner retuned an existing constant or
  field rather than adding struct state. Knobs that add control fields
  consistently failed to earn their keep.

## Recurring failures

- **Loosening retention/admission** (first-reuse promotion, non-zero
  ghost re-admission, freq=1 seed for re-admitted keys): hurts hit-rate
  repeatedly. The baseline's strictness is load-bearing — re-admitted
  keys must re-prove from freq=0, since they've already spent their reuse.
- **Widening the ghost queue:** longer reuse-distance detection re-admits
  stale far-distance keys that displace near-reuse entries, and grows the
  struct. This trace rewards near reuse, not far reuse.
- **Adding struct state:** ghost-sizing control fields grew metadata for
  no benefit; prefer retuning existing fields.
- **Bounding the resident rotation loop:** a fix for a pathology this
  trace lacks — the put tail is driven by the ghost membership scan, not
  main-queue rotation, so capping rotations only sheds proven keys.

## Open questions

- Is workload-a now effectively hit-rate-bound at this capacity, leaving
  only latency-side scan trims? Gains have shrunk to slivers.
- After length (50%), seed-freq (0), and source (incl. main victims) are
  settled, is any ghost knob left that lifts hit-rate?
- Does the main-victim ghost source survive a latency/quality reckoning,
  or is its thin hit-rate gain a wash against its scan churn?
- Are there linear scans beyond the ghost membership walk whose length
  could be trimmed for latency without touching hit-rate?

## Per-policy notes

- **S3-FIFO (only policy tested, only on workload-a):** behavior is
  dominated by two scarce resources — main-queue *room* (split) and the
  ghost *window*. Hit-rate is governed by capacity allocation plus strict,
  reuse-proven retention; latency is governed almost entirely by the ghost
  membership scan on put, never the resident rotation. The hit-rate grid
  (split × freq-ceiling) is saturated; the live frontier is ghost
  *content* (what populates it) rather than length. Treat per-op-time
  movement as noise unless a change actually shortens the ghost scan.
