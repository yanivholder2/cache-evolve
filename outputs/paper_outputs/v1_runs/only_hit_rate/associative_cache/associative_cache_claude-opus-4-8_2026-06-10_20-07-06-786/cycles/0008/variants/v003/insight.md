# Variant v003 Insight

## Helped

(none — the hypothesized hit-rate gain did not materialize)

## Hurt

- hit-rate: Capping the protected segment at `max_size - 1` shrinks each bucket's proven set by one slot; that reserved probationary slot holds a key that never earns immortality, so on workload-a slightly fewer genuinely-hot keys stay resident than when the parent lets the bucket fill entirely with protected keys.
- hit-rate: The withheld-promotion path (counter stays 0, item returns to probationary index 0 on a hit) keeps a repeatedly-accessed key churning in the probationary region instead of locking it in, so under saturation a hot key can still be evicted by a later newcomer — the opposite of the intended protection.

## Takeaway

On this trace, full once-hot immortality (the parent letting newcomers evict even a protected LRU key) beats reserving a permanent probationary buffer: the saturated protected set the hypothesis tried to protect *against* was actually the right size, and trading one protected slot for an un-promotable probation slot costs more retained-hot capacity than it saves from newcomer churn. The "looser/larger protected sizing" open question resolves negative here — shrinking the protected cap, even in its non-eroding form, slightly lowers hit-rate.
