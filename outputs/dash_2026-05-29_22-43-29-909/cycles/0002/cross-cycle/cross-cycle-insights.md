## Recurring wins

- The promotion/retention lever is what reaches and holds the Pareto
  front: across both cycles the single front-holding variant was a change
  to the promotion trigger (recency position, then frequency).
- Frequency/reuse-aware signals are cheap and behavior-compatible — they
  reuse an existing per-item counter, so metadata stays flat while the
  retention logic changes.
- Restructuring the promotion path to an O(1) tail/back-push delivered a
  real latency improvement, the cleanest gain seen so far.

## Recurring failures

- Pure recency as a retention signal is actively penalized on this
  scan/loop trace; biasing promotion toward MRU keeps soon-cold keys
  resident and costs hits. Treat as a dead direction here.
- Item field-narrowing / metadata-shrink has paid off in neither cycle:
  the variants compile and run but the measured metadata-size never moves,
  because the figure is sourced from somewhere other than the overlaid
  struct. Stop retrying this until the metric source is found.
- Swapping an O(1) positional check for an O(n) bucket scan to read
  frequency taxes get latency — gains on retention can be silently undone
  on the latency axis.
- Attribution keeps being the blocker rather than the policy logic:
  hit-rate effects can't be credited when the parent fitness vector lacks
  an entry for the targeted scan/loop trace.

## Open questions

- Where does the measured metadata-size number actually originate? Editing
  Item field widths in the overlay does not change it.
- Can the scan/loop trace be added to the evaluated fitness vector so
  hit-rate changes on the workload these bets target become attributable?
- Can the O(1) tail-push latency win be decoupled from the policy's bucket
  geometry (and from the recency rule it rode in on) to become a clean,
  policy-agnostic promotion-path change?
- Did the second-hit admission gate — meant to keep O(1) back-push while
  filtering scan keys — ever produce a result, or was it never run?

## Per-policy notes

- Scan/loop (LIRS-style) trace: rewards reuse-distance/frequency, punishes
  recency. A strict pre-hit-maximum promotion with ties-never is too
  conservative here — residents share similar counts, so the hot tier
  under-fills; a cheaper threshold (e.g. second-hit admission) is the
  promising refinement.
- The latency win remains tied to one policy's bucket geometry rather than
  generalized across policies; isolating it is still open.
