# Variant v003 Insight

## Helped

- per-op-time: Replacing the multi-row sketch with one splitmix64-mixed hash into a small nibble-packed `Vec<u8>` means one index computation and one cache line touched per estimate/increment, so both get and put latencies dropped.
- metadata-size: The sketch heap footprint shrank by orders of magnitude (two 4-bit counters per byte, table sized to capacity in nibbles), and the containing cache struct got slightly smaller. The headline summary is the Node size, which this change did not touch, so it stayed flat.

## Hurt

- hit-rate: A single hash with 4-bit counters (cap 15) loses the collision resistance of multiple rows and the frequency resolution of wider counters. The admission filter then makes noisier or more tied decisions, and hit-rate fell noticeably on workload-a.

## Takeaway

Collapsing the sketch to one hash with 4-bit counters buys large speed and memory gains, but the cost shows up directly in admission quality. The tight sizing (about one nibble per cached item) probably makes collisions matter, so the hit-rate drop is likely due to both the single row and the small table. A follow-up could keep the packed layout and add a second hash or a larger table, to see how much hit-rate returns for how little speed.
