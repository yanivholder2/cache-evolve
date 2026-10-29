# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants branched from cycle 0006/v002 (fused rotation on the byte-tag base). v001 inlined the ClassicLRU hit path using `rotate_left(1)`. v002 made the BIP period depend on bucket width (1/32 narrow, 1/128 wide). v003 removed the global atomic tick while keeping bucket stride fixed. All three are Winners, but none is a clean speed win. v002 gave the only real movement, a small net hit-rate gain. v003 closed an open question without improving speed. v001 is a negative result that tells us how to do it next time.

## Patterns

- Per-op-time drifted again. Both v002 and v003 got slower on code paths they barely changed or only removed work from, and get-p99 swung in both directions with no code change behind it. Three cycles of this mean single-run tail numbers should not be used to judge anything. Only same-session, repeated runs against the parent count.
- Removing the atomic clock is settled as speed-neutral on the tag/fused base. Bucket stride was pinned (80 bytes) and there was still no put gain. The atomic is uncontended and stays in L1, so it was never the bottleneck. Keep the change for correctness and concurrency only, and stop expecting a speed win from it.
- Generic standard-library helpers can hide slow paths. `rotate_left(1)` on short suffixes falls into a scalar loop that moves one element at a time and cost more than the remove+push memmoves it replaced. The pattern that wins on put is a hand-written `ptr::read`/`ptr::copy`/`ptr::write`, and it hasn't been applied to the hit path yet.
- A static policy split by bucket width was the first hit-rate change in several cycles that did what it predicted. Web_0 on wide buckets improved, and narrow cells stayed bit-identical. But hm_0 on the same wide buckets gave most of it back, so on wide buckets the BIP rate is close to a zero-sum knob between the two workloads, not a free lever.
- Hit-rate invariants again held exactly for every mechanical rewrite. Policy semantics stay robust to restructuring.

## Open questions

- Does an inline, devirtualised hit path with an MRU-skip and a manual `ptr::copy` shift (with the same done for tags) finally lower get p50?
- Is there a wide-bucket BIP period between 1/32 and 1/128 (for example 1/64) that keeps most of the web_0 gain and less of the hm_0 loss? Or is that curve flat?
- Do v002's and v003's apparent slowdowns go away in a back-to-back same-session rerun?

## Continuation

Merge v002's width-keyed BIP and v003's stride-neutral clock removal into one base. Both keep hit rate and size invariant. Re-measure it against cycle 0006/v002 in the same session. On that base, retry v001's hit-path idea with a hand-written `ptr::copy` shift instead of `rotate_left`. Treat the wide-bucket BIP period as a tunable worth at most one more probe, such as 1/64. Drop any further speed expectations from removing the atomic.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007/v003, the clock-free fused-rotation base on byte tags. v001 hand-wrote the hit-path MRU shift with `ptr::copy`. v002 ported width-keyed BIP to this base at 1/64 on wide buckets. v003 replaced the byte tag scan with a SWAR scan. v002 and v003 are Winners. v002 gave a small net hit-rate gain. v003 is a Winner only because its invariants held: get and put both got clearly slower. Neither speed hypothesis produced a speed win.

## Patterns

- Both speed rewrites backfired at p50 on get and put. These are large, same-direction moves, not tail noise. In both cases the rewrite swapped a simple loop the compiler handles well for "clever" code. The variable-length `ptr::copy` probably became an out-of-line `memmove`, and SWAR added fixed per-chunk work plus a branchy mask loop. The put win from cycle 0006 does not generalise. Hand-written moves pay off only when they remove whole memmoves, not when they replace a short inline move.
- Counting compares stopped working as the cost model. The tag pre-filter won in cycle 0005 by cutting full-key compares. Making the byte scan itself wider did not help, which suggests typical scans are short or already vectorised. The remaining miss-path cost is probably not in the tag scan.
- Two caveats on the evidence. The v001 and v003 readings were written without access to the diffs or the generated assembly, so the proposed mechanisms (memmove call, de-inlining) are inferences. And get slowed in the hit-rate-only v002 too, so part of every per-op move this cycle may still be drift.
- The wide-bucket BIP probe is closed. Going from 1/32 to 1/64 to 1/128 moves the web_0 gain and the hm_0 loss roughly in proportion, so there is no sweet spot. The hypothesis's own stop rule fires. Width-keyed masking is still worth keeping as a zero-metadata way to split policy by bucket size: narrow cells stayed bit-identical again.
- Hit-rate and metadata-size invariants held exactly for all three variants. Mechanical rewrites remain semantically safe, so the risk is now entirely in per-op time.

## Open questions

- Does v001's slowdown come from a `memmove` call or de-inlining? Would a fixed-width or const-generic copy, or a fused swap loop, fix it? Is the parent's hit path already near-optimal?
- How far does a typical tag scan actually go before it matches or ends? If the answer is "short", where does the rest of the miss-path time go: the key compare, the rotation, or the bucket indexing and hashing?
- How much of this cycle's latency movement survives repeated same-session runs against the parent?

## Continuation

Adopt v002 (width-keyed BIP, 1/64 wide) as the new base and freeze the policy. Stop tuning the BIP period. Drop SWAR and other hand-widened tag scans. Drop variable-length `ptr::copy` on the hit path, unless an assembly check shows a fixed-width copy would stay inline. The next speed hypotheses should begin by locating the real cost: profile scan length, check codegen, and look at hash/indexing cost. Judge them only with repeated same-session runs against the parent.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Each variant tried to remove a whole unit of work. v001 (from 0007/v003) cached a per-bucket "confirmed absent" result so a put could skip the tag scan. v002 (from 0008/v002) skipped the MRU move on hits in the top eighth of the bucket (lazy promotion). v003 (from 0008/v002) replaced the cache-level SipHash bucket index with one multiplicative hash that supplies both the bucket bits and the tag. All three are Winners. v002 and v003 are the first clear speed wins since cycle 0006: get and put both got faster at p50 and p99, and hit rate stayed well inside budget. v001 counts as a Winner only because its invariants held. Put got clearly slower and get p99 rose.

## Patterns

- Cycle 0008's question about where the miss-path time goes now has an answer: a large share sat above the bucket, in a redundant SipHash per op. Cycles 5–8 tuned in-bucket loops and never reached it. v003's gain is the most credible of the series because it shows up on both ops and both percentiles, and both ops shared the hash it removed.
- "Remove whole work" only wins when the removed work costs more than the state needed to skip it. v002 needed no state and won. v001 added fields, compares and stores to skip a cheap scan that was already in L1, and lost. The tag scan is now cheap enough that it should not be targeted again.
- Lazy promotion worked where earlier hit-path rewrites failed (`rotate_left`, `ptr::copy`). It removes the move for near-MRU hits instead of re-implementing the move. Reordering inside the top window is far from the eviction end, so hit rate barely moved.
- Writes on read paths are costly. v001 made `get_position` store into the bucket, which dirtied the cache line on every miss, and it grew the stride from 80 to 88 bytes. The get-tail rise is consistent with that. Keep lookups read-only and keep the stride pinned.
- Re-mapping buckets with a multiplicative hash on bits 24–55 did not cluster strided block-address keys. Hit rate moved slightly upward. A cheap hash is safe for these traces.
- v002's put gain is partly drift: the put-miss path is unchanged and only put-update benefits. Same-session repeat runs are still required.

## Open questions

- Do v002 and v003 stack when merged? Their effects should be independent (the hash cost above the bucket, the hit-path memmoves inside it).
- Does widening the lazy-promotion window to `n/4` skip more moves without reaching the quarter-depth insertion region, where eviction order starts to change?
- Now that both hashing and hit-path moves are reduced, what is the largest cost left: the full-key compare, the fused rotation on put-miss, or bucket indexing (`%` vs mask)?
- Does stride by itself cost anything? This needs a pad-only control (parent padded to 88 bytes with no behaviour change), and v001 could not isolate it.

## Continuation

Merge v003's single multiplicative hash with v002's lazy promotion into the new base, keep stride at 80, and verify it against both parents with repeated same-session runs. Next, probe a lazy-promotion window of `n/4` with a stop rule at the insertion region, then profile what remains above and inside the bucket. Drop miss memos, any other state added to skip cheap scans, and any writes on the get path. Keep the policy frozen except for the promotion window.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
