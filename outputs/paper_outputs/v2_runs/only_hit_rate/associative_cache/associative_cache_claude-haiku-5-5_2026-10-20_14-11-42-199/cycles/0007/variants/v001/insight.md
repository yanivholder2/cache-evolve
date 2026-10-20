# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Inserting admitted keys at the probationary victim end makes each one the next eviction candidate, so a key that just passed the frequency gate can be displaced on the following miss before it earns a hit, giving back the gain the gate was meant to produce.

## Takeaway

Insert position matters on its own, even with the admission gate in place: moving admitted keys to the victim end of probation lowered hit-rate on workload-a relative to the parent's back-of-slice insertion. The gate does not carry over to this placement, which suggests an admitted key needs some residency before it becomes eviction-eligible; this is measured on one trace at one cache size, so it is not yet general.
