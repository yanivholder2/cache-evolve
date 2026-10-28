# Variant v001 Insight

## Helped

- hit-rate (web_0): In the parent, the main region was a single LRU, so its admission victim was simply the least-recent main key. A key reused many times could still become that victim. Now a key that is hit in probation moves to the 80% protected segment, and the contest victim comes from the probation LRU. Window evictees therefore compete against keys that have not been re-referenced since admission, not against proven-hot keys. On web_0, which mixes scans with a reused working set, this protects the reused keys from scan churn. The gain is about the same at both cache sizes.

## Hurt

## Takeaway

The SLRU gain from cycle 0001 stacks on top of the 1% admission window, and the hypothesis's specific prediction held: almost all of the gain came on web_0, the scan-heavy trace, and almost none on hm_0. hm_0 stayed flat at both sizes, apparently because the parent's single main LRU was already keeping its hot set resident. This suggests that the window and SLRU fix different things. The window fixes admission of newcomers. SLRU fixes which resident key gets sacrificed in the contest. So the two can be tuned independently. This vector measured only hit-rate. The cost of the third list and the demotion that happens on some probation hits (per-op-time) is still unmeasured. The 80/20 split and the u32 sketch with no aging were left untouched, so they remain open knobs.
