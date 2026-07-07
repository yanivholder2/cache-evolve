# Variant v002 Insight

## Helped

- hit-rate: Shifting the SLRU split from 50/50 to 25/75 — growing `prot_cap` from 32 to 48 slots per wide bucket — modestly reduced the rate at which confirmed-hot items were demoted back into the eviction-eligible probationary tier, yielding a small but positive lift in retained working-set hits.

## Hurt

_(no dimensions regressed in the evaluated fitness vector)_

## Takeaway

The 25/75 split confirms the hypothesis direction: a larger protected tier does reduce unnecessary hot-item demotion. However, the gain is negligible, indicating that for this workload the 32-slot protected tier in v003's 50/50 split was already large enough to hold most per-bucket hot sets, and the benefit of the extra 16 protected slots is nearly cancelled out by the corresponding shrinkage of probationary from 32 to 16 slots — a smaller probationary pool has less opportunity to distinguish one-hit wonders from reuse candidates before they are evicted. The segment ratio is not a high-leverage knob on this trace at this cache size.
