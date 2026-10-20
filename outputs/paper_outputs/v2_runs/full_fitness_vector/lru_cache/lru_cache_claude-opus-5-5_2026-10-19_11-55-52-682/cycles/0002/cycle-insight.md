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
