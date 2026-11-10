# Variant v001 Insight

## Helped

- hit-rate: Main-queue entries promoted from small always have `reused = true`, because graduation needs `freq > 1`. So the new `reused` gate in `evict_main()` only filters out keys that came into main through a ghost hit and were never touched again. Those keys stop taking ghost slots and stop earning a second express pass into main. This gave a small gain on web_0 at the large size, the case the hypothesis targeted.
- hit-rate: hm_0 did not just hold steady, it rose slightly at both sizes. The ghost is bounded by `ghost_index.len() >= main_size`, so dropping the never-reused main victims leaves more of that budget for small-queue demotees and reused main victims. In effect the useful ghost history gets longer, rather than the change only removing bad re-admissions.

## Hurt

- per-op-time: The `get` and `put` latencies got worse, the `get` tail most of all. The code change does not explain this. `get` only adds one `bool` store to an entry it is already writing (`freq`), and `evict_main()` now skips `push_ghost` for some victims, which should make `put` cheaper, not slower. The most likely cause is measurement or environment noise, and it should be re-measured before anyone acts on it.

## Takeaway

Gating main-victim ghosting on "hit at least once while resident" is a free filter. The `reused` flag fits in the padding next to `freq`, so `Entry<u64,u64>` is the same size. It improved both traces without hurting either. The result is still weak support for the cycle-2 diagnosis. web_0 at the small size did not change at all, and the large-size gain on web_0 is about the same as hm_0's gain. So unreused ghost re-admissions are at most a minor part of web_0's large-size loss, and the main cause is likely elsewhere, for example small-queue sizing or the ghost bound itself.
