# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0002

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from the baseline, and each tested one axis on its own: residency structure (v001, W-TinyLFU window plus SLRU), sketch freshness (v002, periodic halving with capped counters), and hashing cost (v003, a single cheap hash per key). The winners were v001, which gave the clearest hit-rate gain, and v003, which gave the largest per-op-time gain. v002 also improved hit-rate, but it lost ground on median latency.

## Patterns

- The baseline had two separate weaknesses, and they map cleanly to the two winners. Hit-rate was held back by admission policy (strict admit-or-drop, a sketch that never forgets). Latency was held back by repeated SipHash. Neither winner's fix touched the other's axis, so the two should combine with little conflict.
- Both hit-rate fixes (v001, v002) gained most at the small cache size and much less at the large one. Admission contests at small capacity are where the baseline lost most.
- `get` p99 rose in v001 and in v003, for different reasons: extra region branching and relinks in v001, exposed sketch memory misses in v003. v001 and v002 both improved the `put` tail, but neither could explain why, so treat that as weak evidence. The get tail is now the main per-op-time weak spot across the cohort.
- Metadata cost was flat or slightly better, apart from v001's per-node region byte, which the padding turns into a full word.

## Open questions

- Do v001's window/SLRU and v002's aging add together, or does the window already absorb most of the stale-hot-key problem?
- Is v003's get-tail rise real, or benchmark noise? Would a cache-line-local sketch block fix it?
- Can v001's region tag be packed into spare bits to win back the per-node bytes?

## Continuation

Next cycle should start by merging v001's W-TinyLFU structure with v003's single-hash fast path. Then layer v002-style aging on top, stored as packed 4-bit counters so it costs no extra heap or sweep work. After that, target the get tail with a cache-line-blocked sketch, and remove v001's region byte. Do not try aging on its own again over the u32 counter layout. Its latency cost is already known.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
