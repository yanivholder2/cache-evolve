# Variant v003 Insight

## Helped

- metadata-size: The headline `Node<u64,u64>` stays at 16 bytes because the doorkeeper was put inside `CountMinSketch`, not in the per-entry node. That was the constraint the hypothesis set, and it held.

## Hurt

- hit-rate: The doorkeeper did not sharpen the duel, and the hypothesis's own refutation bar (below the parent) was crossed. The filter is only `width` bits wide, and two bits are set per new key over a reset period of `16 * capacity` increments. That is long enough for many distinct keys to accumulate, so the bit array probably fills up and starts passing one-hit keys into the counters anyway. Each periodic clear also takes the doorkeeper `+1` away from every recurring resident at once. Meanwhile, the next arrivals get it back on first sighting, which adds noise to exactly the newcomer-vs-victim comparison the change was supposed to clean up.
- per-op-time: Every `get` now goes through `increment_hash`'s doorkeeper path. That means two more random loads into a separate bit array, a hard-to-predict membership branch, and an `ops` tick. The first-sighting branch still calls `estimate_counts`, so it loads all four counters anyway. The doorkeeper skips the counter stores but none of the loads, so the change only adds work, raising both the `get` and `put` medians and tails.
- metadata-size: Outside the headline number, the sketch heap grows by the doorkeeper's bit array. The inline `TinyLfuCache` also gets bigger, because the sketch now holds another `Vec` header plus `dk_mask`, `ops` and `reset_period`.

## Takeaway

On this workload, a doorkeeper in front of a sketch that never decays costs time and does not help admission. With width factor 8 and a `16 * capacity` reset period, the filter is too small for the volume of one-hit keys it has to absorb. Its periodic clear also acts as a crude, all-at-once aging step, the same kind of thing the run already found harmful when the sketch was halved. If doorkeepers are tried again, the filter needs its own sizing (more bits, or a reset period tied to the bit-fill level), and the first-sighting path should not load counters at all. Otherwise the change cannot pay for itself on latency.
