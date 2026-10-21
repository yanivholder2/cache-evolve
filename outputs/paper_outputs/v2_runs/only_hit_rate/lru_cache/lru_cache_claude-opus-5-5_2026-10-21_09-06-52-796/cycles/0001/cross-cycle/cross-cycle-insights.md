## Recurring wins

- Only one cycle so far, so nothing has recurred yet. Treat what follows as early signal, not established pattern.
- Admission filtering (S3-FIFO-style probation and ghost tracking) is the only lever that has raised hit rate so far. The gain came on the scan- and loop-heavy workload.
- Coarse recency (CLOCK second-chance) kept LRU's hit rate. Strict LRU ordering looks dispensable for hit rate.

## Recurring failures

- No repeated failure yet. One early negative: changing how recency is tracked (CLOCK) or only changing data layout (hasher, narrow indices) did not improve hit rate. Expect hit-rate gains from admission, not from recency precision.
- Measurement gap: `metadata-size` and `per-op-time` were not reported. Memory and latency claims stay unverified. If this happens again, it is a harness problem to fix before any more variant work.

## Open questions

- Does the ghost-queue and ghost-set overhead in admission-filtering policies cancel the memory saved by dropping `prev`/`next` links?
- Are hit-path operations actually faster once recency links are removed or narrowed?
- Does CLOCK's victim sweep cause `put` tail-latency spikes when most entries are marked referenced?
- Can policy-independent layout changes (fast hasher, `u32` indices, MRU fast path) be stacked onto a winning policy for a pure speed gain?
- How sensitive is S3-FIFO hit rate to the size of the small probation queue?

## Per-policy notes

- **S3-FIFO (admission filtering):** current best on hit rate and the default parent for the next cycle. Its memory and speed are unmeasured, and the ghost structures are the main risk to watch.
- **CLOCK:** matches LRU's hit rate with less ordering state. It is only worth continuing if measurement shows a clear memory or latency win. Watch for sweep tail latency.
- **Strict LRU + representation changes:** hit rate is unchanged, as designed. Its value is purely in memory and speed, which have not been measured yet. It is a candidate for grafting onto other policies.
