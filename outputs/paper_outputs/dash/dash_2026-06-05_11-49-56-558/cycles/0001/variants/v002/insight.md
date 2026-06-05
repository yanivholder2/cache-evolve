# Variant v002 Insight

## Helped

- hit-rate: Replacing LRU with LFU at the bucket level protected frequently-accessed items from being displaced by recently-inserted but rarely-reused items, yielding a meaningful improvement on the frequency-skewed workload-a trace.

## Hurt

_(none)_

## Takeaway

Forcing LFU eviction unconditionally in `new_bucket` — discarding the caller-supplied policy — demonstrates that the workload-a trace has a stable hot set that LRU's recency bias was actively harming: newly inserted cold items were pushing hot items out before they could be re-requested. LFU's per-item counter, which persists across stash→target→probing tier moves, compounds its advantage the longer high-frequency items survive, making this change particularly effective for skewed access distributions.
