# Variant v001 Insight

## Helped

## Hurt

- hit-rate: On a probing-bucket hit, `promotion_get()` now removes the key from its probing bucket and re-inserts it at the MRU end of its target, so a key that had already earned probing residency has to re-climb through target-bucket competition to regain it, and the reinsertion can evict a target resident into the demotion cascade.

## Takeaway

The change turns a probing hit from an in-place touch into a removal and reinsertion, so the probing tier stops holding the keys that proved hot there. The hit-rate drop on workload-a indicates that retaining probing residency matters more than the MRU placement in the target bucket that this change buys.
