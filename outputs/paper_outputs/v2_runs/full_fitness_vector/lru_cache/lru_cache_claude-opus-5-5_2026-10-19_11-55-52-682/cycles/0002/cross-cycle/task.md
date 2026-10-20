# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched off the baseline and each attacked strict LRU in its own way: SLRU segmentation (v001), a CLOCK reference-bit ring (v002) and BIP head insertion (v003). All three became Winners, and none of them got worse on any measured dimension except v003, whose metadata cost grew by one fixed per-cache word.

## Patterns

- Strict LRU recency order is a liability on the LIRS workload-a trace. Each of the three departures from it gained hit-rate: second-touch promotion, second-chance bits and loop-resistant insertion. That includes CLOCK, which was expected to lose a little.
- Per-op-time improved together with memory locality rather than with shorter code paths. v001 got smaller nodes, v002 stopped relinking on a hit, and v003 concentrated churn on a few hot slots. All of them sped up get and put at both p50 and p99.
- Node layout is the real lever for metadata-size. Removing links (CLOCK) or narrowing them to `u32` indices (SLRU) shrank the node and probably also helped speed. BIP left the layout alone and left metadata flat.
- The mechanisms are orthogonal and could stack: insertion position (BIP), a hit path with no relinking (CLOCK), and admission/protection segments (SLRU).

## Open questions

- Does the hit-rate ranking hold at other cache sizes and on traces other than workload-a? Every result so far comes from one trace at one size.
- How does CLOCK's put tail behave when most of the ring is referenced at once, forcing long sweeps?
- How much of v003's speed gain is real and how much is benchmark noise?
- Do the gains combine or overlap? All three variants target the same problem: a loop or scan flushing the reused keys.

## Continuation

Next cycle should try combinations instead of more single-axis rewrites. Good candidates are CLOCK plus a probation/protection split (CLOCK-Pro-like or a 2-bit/segmented clock), BIP-style insertion into SLRU probation, and applying the `u32`-link compaction to any policy that keeps lists. Add a check on other cache sizes or workloads and a test of the CLOCK sweep tail, so we don't overfit to workload-a at this one size. Drop further work on plain LRU with only tweaks to the order it keeps.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle followed cycle 0001's advice to combine mechanisms. It tried BIP insertion inside SLRU probation (v001) and a 2-bit GCLOCK counter on CLOCK (v002), and added one fresh single-queue policy, SIEVE from baseline (v003). All three are listed as Winners, but they did very different things. SIEVE improved on all three dimensions. GCLOCK traded put-tail latency for hit-rate. BIP inside SLRU got slightly faster but lost hit-rate.

## Patterns

- Scan protection mechanisms don't add up. SLRU probation already keeps one-touch keys away from reused keys, so layering BIP on top only shrank the window in which new keys can earn a second touch. On this trace, "make new keys the immediate victim" works against admission windows rather than with them.
- Quick demotion of new keys plus lazy promotion of reused keys (SIEVE) reached the best hit-rate so far with one queue and one bit. A simple policy matched or beat the multi-segment one.
- A relink-free hit path and `u32`-compact nodes keep paying off on get latency and metadata. Every hand-based variant (GCLOCK, SIEVE) kept a cheap get.
- Hand sweeps are now the main cost on the put tail. GCLOCK's deeper per-slot credit made put p99 visibly worse. SIEVE's speedup was also smallest at put p99 because of long runs of visited nodes. This answers cycle 0001's CLOCK-sweep question: the tail cost is real.
- Reusing padding bytes (`freq: u8`, `visited: bool`) adds policy state for free, so metadata pressure now comes only from link width.

## Open questions

- Can the sweep be bounded without losing the frequency or visited signal? Options include a smaller `FREQ_MAX`, decrementing by more than one per pass, or a cap on the number of steps per insert.
- Does SIEVE's narrow hit-rate lead hold at other cache sizes or traces? Every result is still from workload-a at size 2048.
- Would SIEVE gain from a small frequency counter (a SIEVE/GCLOCK hybrid), or would that repeat GCLOCK's tail cost?
- Would resizing probation or an admission filter beat SLRU's current split, given that head insertion did not?

## Continuation

Use SIEVE (v003) as the main parent. Next cycle should either bound its hand sweep to fix the put tail, or test whether a 2-bit visited counter adds hit-rate without the tail cost GCLOCK paid. Drop BIP/head insertion inside SLRU probation and further single-bit CLOCK tweaks. If SLRU is revisited, change the probation size or admission rather than the insertion position. Add at least one other cache size or trace before trusting SIEVE's narrow lead.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
