# Variant v001 Insight

## Helped

- per-op-time (put): Put p50 and p99 both dropped sharply. That is the largest movement in this variant, and it came on the op the hypothesis did not target. `put_lip`'s update branch goes through `get_from_position`, so with the new early return, re-puts of near-MRU keys skip the dynamic `eviction_policy.get_from_position` call and the `tags.remove`/`tags.push` memmoves. The miss path is the same as v003's (same fused evict+insert, same BIP counter). So the only code-level explanation is the skipped update-path promotion, which apparently happens often in the put benchmark.
- per-op-time (get p50, the summary): Get p50 went down, but only by the smallest step the measurement can show. The hypothesis's literal "below v003" condition is met. The rationale's "clearly below" is not. Removing two memmoves and a virtual call on top-eighth hits gave almost nothing once v003 had already removed the per-op SipHash.

## Hurt

- per-op-time (get p99): Get tail latency rose, even though the skip removes work. The new branch (`position + (n >> 3) >= n - 1`) is data-dependent and sits on every lip hit. It probably mispredicts when hits alternate between the top eighth and deeper positions. The removed memmove was cheap on the hits that matter most for the tail, so it bought too little to cover that cost.
- hit-rate: A small drop, inside the ±0.0005 band and the same size the hypothesis expected from v002. It appears only in the two 131072-size cells. Small caches did not move. This fits a lazy-promotion window that grows as `len/8`: in wider buckets, top-eighth items that are left in place drift further toward eviction before they are refreshed.
- metadata-size: unchanged. `LAZY_PROMOTE_SHIFT` is a const, there are no new fields, and the 80-byte bucket-stride test still passes.

## Takeaway

Stacking lazy promotion on top of single-hash v003 did not add up on get. Once the SipHash is gone, the move-to-back on near-MRU hits is not a big enough share of get time to pay for an extra data-dependent branch on every hit. The result is a negligible p50 gain and a worse p99. The real payoff of the skip shows up in the put-update path, where it removes the same promote work. Future get-side work should therefore stop targeting the in-bucket promotion. If the skip is kept, its value should be judged by the put latency it saves, not by get latency.
