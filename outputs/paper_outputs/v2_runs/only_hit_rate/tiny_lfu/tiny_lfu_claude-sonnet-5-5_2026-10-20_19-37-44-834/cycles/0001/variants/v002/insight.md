# Variant v002 Insight

## Helped

- hit-rate: Only a marginal gain over baseline. Sending every new key through a ~1% window and a probation/protected main region lets re-referenced keys gain protection, and the sketch-gated window-to-probation duel keeps scan-like one-hit keys from displacing them. The effect is tiny on this trace.

## Hurt

- metadata-size: Not reported in this variant's fitness vector, so the effect is unmeasured. The change adds a `seg: u8` tag to `Node`, which is likely to grow it through padding. The three head/tail/len arrays also add a small fixed cost to the cache struct.
- per-op-time: Not reported either, so the effect is unmeasured. Promotion and demotion between segments add extra unlink/push_back list operations, and each full `put` now computes two sketch estimates plus a window-to-probation move. Cost per operation is probably higher than the single-list baseline.

## Takeaway

Adding a window and segmented main region to the baseline's already sketch-gated LRU gave almost no hit-rate gain on workload-a at size 2048. Baseline admission had already captured most of the benefit, so the extra structure mainly adds complexity and probable per-op and metadata cost. Because the variant's vector omits those two dimensions, the trade-off cannot be confirmed from the data. Any follow-up should restore their measurement before judging this design.
