# Variant v002 Insight

## Helped

- hit-rate: hit rate rose clearly above v003's. The reason is that both segments of the merged bucket now act as one shared SLRU instead of two hash-selected LRU halves. A key that misses enters probation at index `len0`. Only a second reference promotes it into protected, which holds 3/4 of the slots. One-shot keys from the LIRS loops and scans therefore cycle through probation and are evicted from `items[0]` without displacing re-referenced items. The change also removes v003's cross-half interference, where a hot half could push demoted items into its partner and the "evicted guest is always dropped" rule threw items away. Every key can now use the whole `2 * bucket_size` slots, and a get never drops an item.

## Hurt

- per-op-time: get and put both got slower at p50 and p99. Put's p99 slowed the most. The SLRU moves cost more data movement than v003's half-local moves:
  - A probation hit does `items[pos..].rotate_left(1)` across the entire protected segment, which can be up to 3/4 of the bucket.
  - `pair_get` always scans the whole merged bucket, where v003 could stop at the home half.
  - A miss on a full bucket costs two shifts, each spanning most of the bucket. The first is `remove_at(0)`, which shifts the whole `Vec` left. The second is `items.insert(len0, ..)`, which shifts the protected segment right.

## Takeaway

On workload-a, recency within each bucket was not the problem. Scan pollution was. Keeping segment membership in the `len0` boundary v003 already had makes the cache scan-resistant at no metadata cost: `Item` is unchanged, and SLRU needs no per-item flags or timestamps. The cost is latency, because both promotion and eviction from the front are memmoves sized to the protected segment. A follow-up that keeps the SLRU semantics should try to make those moves cheaper. Two options are a ring or offset representation, so that evicting `items[0]` needs no shift, or putting protected first so new items are appended at the tail. The 3/4 protected split is the hypothesis's starting value, not a tuned one.
