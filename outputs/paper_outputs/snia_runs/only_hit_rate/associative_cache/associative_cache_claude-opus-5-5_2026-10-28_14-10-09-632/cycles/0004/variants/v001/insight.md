# Variant v001 Insight

## Helped

- hit-rate (`web_0`, mainly at 131072): On `web_0`, most MRU samples are one-touch blocks. Each of them dies unhit and costs a PSEL decrement, and none of them can earn a reward. So the 1024-slot buckets drift into the `psel <= 3` pure-LIP band, and most of the pollution v002 added there goes away. The per-item tag in `lfu_counter` works as designed on this side: because the tag lives until the item is evicted, the negative signal that cycle 0003's ring lost is now delivered. `web_0` at 16384 improved a little for the same reason.

## Hurt

- hit-rate (`hm_0`, mainly at 16384): The selector also pushed the 128-slot `hm_0` buckets away from v002's period of 32, and most of v002's gain was lost. The feedback rule is biased against sampling in three ways:
  - **Biased start.** PSEL starts at `PSEL_MAX / 2 = 7`, which is the period-128 band. Every bucket therefore starts at a quarter of v002's rate and must show net-positive evidence before it gets back to period 32.
  - **50% break-even.** The update is a symmetric ±1 per sample. PSEL climbs only if more than half of all samples become late hits. Sampling can be a net win with a much lower success rate, because one rescued long-reuse block can pay for several dead samples.
  - **Strict reward test.** A reward needs `age > probe_len`, with age counted in admissions only. Hits that promote items from below the probation point also push a probation item toward eviction, so cold insertion loses blocks sooner than `ins` misses. Some hits that only MRU placement saved are therefore classed as neutral, and their tags are cleared without credit.
- hit-rate (`hm_0` at 131072): A small loss, consistent with the same downward drift removing the few useful samples in 1024-slot buckets.
- (`per-op-time` was not measured, even though the hypothesis asked for it. The change adds one tag read on hits, one tag check on evictions, and a small `match` per admission. It adds no scans and does not change the size of `Item`.)

## Takeaway

Storing a per-item tag in the unused `lfu_counter` fixes the lost-signal problem: unhit samples are now reliably penalised. But a symmetric ±1 PSEL that starts mid-range votes on whether most samples hit, not on whether sampling pays off. It therefore leans toward LIP everywhere and gives back `hm_0`'s gain to recover `web_0`. The trade almost cancels out, so the summary is slightly below v002. The selector needs a weighted update (a larger increment per credited hit than the decrement per dead sample, or a start at the period-32 band). It also needs a reward test that counts promotions as well as admissions. Without both, this feedback cannot keep sampling where it is a net win.
