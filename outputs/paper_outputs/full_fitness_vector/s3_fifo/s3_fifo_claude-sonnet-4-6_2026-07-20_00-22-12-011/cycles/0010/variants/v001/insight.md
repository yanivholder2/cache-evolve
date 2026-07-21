# Variant v001 Insight

## Helped

- metadata-size: Collapsing `data`, `freq_map`, and `ghost_set` into a single `state_map: HashMap<K, Option<(V, u8)>>` removed three separate map/set headers from the struct, shrinking `S3Fifo` by 96 bytes (256 → 160).
- per-op-time: Replacing the two-probe put path (`data.contains_key` + `ghost_set.contains`) with a single `state_map.get()` call, and eliminating `ghost_set.remove()` on ghost hits, halved put p50 and put p99 while leaving get latency unchanged.

## Hurt

## Takeaway

Encoding ghost membership as a `None` sentinel in a unified state map is a pure win for this workload: the two-probe pattern on every `put()` was the dominant cost, and collapsing it to one lookup halved put latency without touching hit-rate or admission policy. The struct-size reduction is a bonus from eliminating two entire map headers, not the primary driver of the latency gain.
