# Variant v003 Insight

## Helped

- **metadata-size**: Dropping `timestamp: Instant` from `Item` collapsed the struct from three fields (key 8 B + value 8 B + lfu_counter 8 B + timestamp 16 B) to two meaningful fields plus the counter, landing at exactly 24 bytes as the hypothesis predicted.

## Hurt

- **per-op-time**: Get-path latency rose even though the struct shrank and ClassicLRU never touched the removed field; the smaller `Item` layout likely changed how entries pack across cache lines within each bucket's `Vec`, producing a net slowdown for the get hot-path despite put latency improving.

## Takeaway

Removing an unused struct field is a reliable way to win on metadata size — the 40-byte → 24-byte reduction landed precisely because `timestamp: Instant` was genuinely dead weight for the ClassicLRU evaluation path. However, shrinking a frequently-iterated struct is not a free performance gain: the changed memory layout altered cache-line utilisation enough to hurt get latency, warning that struct-size and op-time optimisations can pull in opposite directions even when the removed field is never read.
