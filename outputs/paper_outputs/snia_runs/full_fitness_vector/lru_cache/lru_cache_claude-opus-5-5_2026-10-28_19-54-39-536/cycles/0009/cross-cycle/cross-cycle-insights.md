# Cross-Cycle Insights (cycles 0001–0009)

## Recurring wins

- **Compact `u32`-index nodes with policy state folded into existing bits** (link high bits, padding byte). These have been the default base since cycle 2. They keep metadata small with no hit-rate cost, and they made SLRU affordable.
- **Flag-packed SLRU** has been the policy base since cycle 2. It is the only policy that raised hit-rate on every trace at both sizes.
- **Put cost tracks hash and bookkeeping count.** This is the most consistent rule, holding across cycles 3–9. Removing a hash per put made put faster: single-`Entry` no-clone put (c4), cheap Fx ghost tags with ghost work only on the vacant path (c6), and a denser ghost table (c9).
- **Selective, gentle reprieves beat a uniformly wider window.** Fingerprint-ghost second chances that stay in probation gave the only net hit-rate gains after cycle 2 (c4, c5, c7, c8). A static probation retune always traded hm_0 against web_0.
- **Size-gating or size-tuning the ghost** removed the large-size losses. At 131K, ghost width works as a reuse-distance filter: capacity/8 is too short, capacity/2 lets in hm_0 loop keys, and capacity/4 is slightly best. Width tuning is now essentially used up.
- **Zero-seeded or deterministic ghost hashing** made small hit-rate deltas trustworthy (from c7 on).

## Recurring failures

- **Get p50 resists every targeted lever.** Tried so far: list and relink removal (c1, c3, c5), a cheaper hasher (c2), a custom index table (c6), forced inlining or a cold split (c7), and ghost shrinking (c8–c9). It also moves on byte-identical get code (c5–c9). Single-run get-p50 and get-p99 deltas look like noise or layout/codegen sensitivity. The ghost-footprint theory from c8 was put in doubt by c9.
- **Any extra work on the hit path** (ticks, masking `next`, extra AND/flag ops) made get p50 worse. Keep `on_hit` and the header's hot cache line untouched.
- **Faster or cheaper hashers and custom map tables**: get never improved, and the custom table hurt the put tail through rehash-on-shift.
- **Aggressive history and admission**: full-key ghosts that go straight to protected, and miss-only tie-rejecting TinyLFU gates (c3). Both hurt web_0.
- **Online hill-climbing of the SLRU split** (c4) and **static probation-share retuning** at either size (c5, c8). These chase noise or trade hits between traces.
- **CLOCK/GCLOCK variants**: aging on the put path inflated sweep cost, and protected-segment CLOCK did not lower the median (c2, c5).
- **Pre-eviction-history reprieves** ("was once protected" flag, c9) kept cold keys and hurt hit rate. Only a re-reference seen after eviction predicts reuse.
- **Remove-before-insert via a second hash** (c7): the extra SipHash cancelled the saving. It can only be retried through a path that keeps the hash.
- **Bitset ghost with per-put cursor aging** (c9): it used fewer bytes but put cost more.

## Open questions

- What is the real noise floor for per-op timing? Paired, repeated parent/child runs are still missing after many cycles, and no latency claim below a few percent is confirmed.
- At 131K, does the ghost still pay for its bytes and put cost compared with no ghost?
- How else can ghost-hit reprieves be granted? For example: stricter criteria at large sizes, the reprieve's destination, or repeat-reprieve limits. This is the remaining hit-rate lever, as opposed to window size.
- Can a bitset or Bloom ghost age itself without any per-put work (generation flip or double buffer)?
- Should metadata-size count the heap ghost table, which is a large hidden cost at 131K?
- web_0 at 131K is only partly solved, and the right amount of admission aggressiveness remains trace-sensitive.

## Per-policy notes

- **LRU (baseline):** list maintenance is not the cost driver for get. The map probe and layout dominate.
- **SLRU:** the main line from c2 on. The current base is the c9 v001 lineage: single-`Entry` no-clone put, `prev`-bit reprieve, Fx 8-bit-tag ghost at capacity/4 at 131K, and a size-gated ghost at 16K. Its costs sit in put and eviction, not get.
- **SLRU + ghost reprieve:** helps both traces at 16K. At 131K it is net positive only with a correctly sized window. Tag collisions at 8 bits now have a visible but marginal hit-rate cost.
- **CLOCK / GCLOCK:** lazy promotion cut metadata but gave only modest, size-dependent hit-rate gains. Put tails grow with sweep length. It has been abandoned since c5.
- **TinyLFU / A1out-style admission:** failed in the bare forms tried. A W-TinyLFU with a cheap hash and hit counting has never been tested.
- **Adaptive split:** abandoned. It reacts to noise faster than to the workload.
