# Variant v002 Insight

## Helped

- hit-rate: No dimension improved; seating promotions at the protected-LRU
  boundary did not shield hot keys in a way the workload rewarded.

## Hurt

- hit-rate: Inserting a freshly promoted key at the first-protected-slot index
  (`items.insert(b, item)`) instead of the MRU back demotes it relative to the
  whole protected stack, so on this workload promoted-but-still-hot keys sit
  near the protected LRU end and get evicted (fall-back index 0) sooner than
  under the parent's vault-to-MRU promotion, costing hits.

## Takeaway

For this trace, a single hit is a reliable enough signal that a promoted key
deserves full MRU standing: making the protected region a true second LRU stack
that forces re-earning actually penalizes genuine rising keys more than it
shields established ones, so the parent's promote-straight-to-MRU placement is
the better policy.
