# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Making `ghost_size` a self-tuning second controller let it shrink the ghost as low as `ghost_lower() == capacity/2`, well below the parent's pinned `main_size` (≈23/24·capacity), so on every window where the conversion rate dipped into the ≤10% shrink band the policy threw away more remembered eviction history than v001 ever did — discarding exactly the re-admission signal the change was meant to capture.

## Takeaway

Sizing the ghost to its own measured productivity backfired because the controller's floor undercut the parent's static budget: a windowed conversion rate that occasionally falls below 10% triggers a shrink toward `capacity/2`, and the lost ghost depth costs more hits than the rare grow-toward-`2·capacity` excursions recover. The cycle-0008 framing does not extend cleanly to ghost capacity here — ghost history on workload-a is hit-productive enough that pinning it near `main_size` beats letting a feedback loop trade it away.
