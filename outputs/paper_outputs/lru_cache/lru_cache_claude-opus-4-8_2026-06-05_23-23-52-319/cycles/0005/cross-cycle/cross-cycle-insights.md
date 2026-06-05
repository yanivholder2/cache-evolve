## Recurring wins

- Eviction *ordering*, not extra residency, is the dominant lever: every
  win held capacity fixed and gained purely by choosing which resident
  dies first.
- "Protect on second access" is the core signal — distinguishing
  one-hit-wonders from genuine reuse repeatedly converts directly to hits
  on this reuse-heavy trace.
- Enlarging the protected tier paid off across multiple consecutive
  cycles (80/20 to 90/10 to 95/5), each step beating its parent — though
  the marginal gain shrank every time.
- Reuse on this trace arrives fast (within the narrow probationary
  window), so first-hit promotion is confirmed correct.

## Recurring failures

- Loosening the promotion signal (wider probationary, two-hit gates)
  consistently regresses: fast reuse means stricter/slower gates just
  shed keys before their second touch.
- Non-recency admission/victim signals (W-TinyLFU window, LFU victim
  selection, frequency at demotion) keep losing inside the SLRU frame —
  they mistrack recency or evict genuinely-reused keys; this trace is not
  one-hit-wonder dominated.
- Ghost / reuse-distance re-admission is directionally right but
  structurally redundant against a maximally-protected tier, and costs
  extra structure plus per-insert scans; tested in isolation and layered,
  both inert.

## Open questions

- Has the protected-ratio lever hit its ceiling at 95/5, or is there a
  sliver left before the probationary floor breaks the eviction path?
- The canonical LIRS rebuild reported a real hit-rate lift but did not
  land in the Winners — was the gain within noise, or did it trip a
  contract/invariant check? Must be resolved before trusting it.
- If the LIRS gain is real, does it dominate the SLRU incumbent outright
  or only on this LIRS-derived home-turf trace?
- Is any orthogonal workload signal left, or is the recency/promotion
  axis fully consumed?

## Per-policy notes

- **Pure LRU**: baseline; loses by evicting reused keys as readily as
  one-hit-wonders. Dropped early.
- **SLRU (segmented)**: the durable parent across cycles. Binding
  constraint is protected-tier residency (then the probationary floor),
  not promotion timing or intra-segment order. Geometry now near-exhausted.
- **CLOCK / midpoint insertion**: cheap single-list approximations of the
  protection signal; helped initially but never beat explicit segmentation.
- **LFU / frequency-based**: regresses on this trace at high protected
  ratios — frequency values tie and it just mistracks recency.
- **LIRS**: the one genuinely off-axis mechanism to report a real gain;
  unresolved (reported lift did not win). The live thread to re-run cleanly.
