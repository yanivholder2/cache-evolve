# Variant v001 Insight

## Helped

## Hurt

- **per-op-time**: `maybe_adapt()` is called unconditionally at the top of every `get()` and `put()`, performing a counter increment and a branch on each operation; this overhead raised the p50 from 0 ns (parent) to 100 ns even in epochs where no adaptation fires.
- **metadata-size**: Adding `demotion_count` and `op_count` to `LruCache` grew the struct from 120 to 128 bytes, though the `Node<K,V>` summary metric held at 24 bytes and the fitness score is unchanged.

## Takeaway

The adaptive quota mechanism avoids permanently narrowing `prob_cap`, but the cost of counting every operation is not amortised: the per-op overhead is paid on each call regardless of whether an epoch boundary is reached. On this workload the demotion rate never exceeded the `capacity/16` threshold during any epoch, so `prob_cap` stayed at its initial `capacity/4` value throughout — identical to the parent's fixed policy — yet the instrumentation alone was enough to eliminate the parent's per-op-time advantage. Adaptive sizing only pays off if the saved policy quality outweighs the unconditional per-call bookkeeping cost; here neither side of that trade moved.
