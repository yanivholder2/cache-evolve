# Variant v003 Insight

## Helped

- hit-rate: The two-row frequency sketch makes `put()` reject a new key at capacity when its estimated frequency is below the LRU victim's, so one-hit scan keys no longer displace the resident working set on the LIRS workload-a trace.

## Hurt

- per-op-time: `record()` runs on every `get()` and `put()`, and each call hashes the key twice through `DefaultHasher` (SipHash, row seed hashed first). `put()` at capacity adds two more `estimate()` hashes per candidate and victim, so the `get` p50 regression comes from the hash-per-access cost the admission check introduces.
- metadata-size: The `LruCache` struct grew because it now carries the sketch (two `Vec<u8>` headers, a mask, and the sample counters). `Node` is unchanged, so the summary dimension is flat, but the cache-wide footprint rose.

## Takeaway

Gating admission is a different lever from reordering: this variant leaves the list and `get()` ordering alone and changes what enters the cache, and that is where the hit-rate gain comes from. The cost is paid per operation rather than in memory, because the sketch is hashed on every access. The gain is measured on one trace at one capacity, so it does not yet show that the admission filter helps beyond the scan pattern in workload-a.
