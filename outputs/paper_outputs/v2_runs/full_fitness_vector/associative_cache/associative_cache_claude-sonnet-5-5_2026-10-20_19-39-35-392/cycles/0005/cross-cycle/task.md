# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-axis changes to the baseline were tried: wider buckets (geometry), partial-promotion LRU (policy), and a rotate-based move-to-back (micro-optimization). None produced a Winner. Hit rate stayed flat or dropped, and get latency stayed flat or got worse.

## Patterns

- Hit rate looks insensitive to associativity: 4x wider buckets gave only a marginal gain, so conflict misses are probably not the bottleneck on workload-a.
- Weakening recency (partial promotion) cost hit rate. In small buckets a not-fully-promoted hit item becomes the next victim.
- The hit path is not dominated by item movement. Replacing remove+push with rotate gave no get speedup, and the p50/p99 shifts look like noise.
- Put latency moved favorably in all three variants, including the two that never touched the put path. This suggests a measurement artifact or shared-path effect, so don't treat it as a real win.
- Position-based tweaks to LRU and geometry are low-yield. Metadata size never changed because no variant added state.

## Open questions

- Is capacity or the eviction policy itself the real hit-rate limit? A policy using frequency or a reference bit is untested.
- Why did wide buckets make puts cheaper? The cause is unexplained.
- Where does get time actually go (hashing, lookup, dynamic dispatch)? No variant measured it.

## Continuation

Drop geometry sweeps and rotate/memmove micro-optimizations; both are known dead ends here. Next cycle should try policies that carry a small extra signal (frequency counter, reference bit, or a ghost/admission filter) for hit rate. For get latency, target hashing, bucket lookup or dispatch overhead rather than the item move. Discount put-latency gains until they reproduce.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants targeted scan resistance in the eviction/insertion policy: a CLOCK reference bit, a 2-bit frequency counter with LRU tie-break, and low-position insertion (LIP/BIP-style). All three became Winners with hit-rate gains, in contrast to cycle 0001's position-only tweaks. The insertion-position variant showed the clearest gain.

## Patterns

- Policy changes that keep one-hit-wonders from displacing reused entries are what moved hit rate. Capacity and associativity were not the limit.
- Gains were small to clear, and they ranked by how little the change cost. Stateless insertion position and the reference bit beat the frequency counter.
- Latency followed bookkeeping cost. The reference bit made put and the get tail cheaper by removing reordering. The frequency counter added work on every hit and put, which slowed both slightly.
- Median get stayed flat wherever the hit path got cheaper. This confirms that item movement isn't what dominates get time, so the lookup path is the remaining suspect.
- No variant grew metadata. The extra state fit into existing padding or wasn't needed.
- The v003 latency improvement is unconfirmed and may be noise or come from wide buckets. Treat it with the same caution as cycle 0001's put-latency artifact.

## Open questions

- Do the policy gains stack, for example low-position insertion plus a reference bit?
- Is the frequency signal redundant once scan resistance exists, given its marginal gain and latency cost?
- Where does median get time go (hashing, bucket lookup, dispatch)? Still unmeasured.
- Do these gains hold on workloads other than workload-a, or are they specific to its scan-heavy traffic?

## Continuation

Combine the cheap scan-resistant mechanisms (low-position insertion with a CLOCK reference bit) and drop the per-slot frequency counter, since its latency cost outweighs its small gain. Aim the next latency hypotheses at the lookup path (hashing, bucket search, dispatch) rather than eviction or item movement. Re-check any latency gain that comes without a mechanistic reason.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Variants explored the insertion position and promotion granularity: LRU-end insertion with a rare hashed MRU escape (v001), a CLOCK reference bit stacked on low-position insertion (v002), and mid-bucket insertion with transpose-by-one promotion (v003). v001 and v003 were Winners, but the evidence for v001 is weak, since its own insight reports hit rate and latency as slightly worse than the parent. v002 was the only clear loser.

## Patterns

- Stacking scan-resistance mechanisms did not compound. The reference bit combined with low insertion lost both hit rate and speed, which contradicts the cycle 0002 hope that the cheap mechanisms would add up.
- Insertion depth has a sweet spot. Pushing toward the LRU end starves new entries of the chance to prove reuse. A middle or len/4 position keeps scan resistance and still lets reused entries survive.
- Explicit promotion on hit matters when insertion is low. Proven entries have to be moved up, and a one-bit second chance is too coarse to do that.
- Latency again followed added per-access logic (hash-conditional branches, bit masking, mid-bucket shifts). Get time did not improve in any variant, which is further evidence that reordering is not what dominates it.
- No variant changed metadata size.

## Open questions

- Are the Winner status and hit-rate edge of v001 real, or within noise? Its insight describes a marginal loss.
- Can mid-insertion keep its hit-rate gain without the median put cost of shifting half the bucket?
- Where does median get time go (hashing, bucket lookup, dispatch)? This has been unmeasured for three cycles.
- Do any of these gains hold on workloads other than workload-a?

## Continuation

Stop varying insertion position and stacking reference-bit schemes; the position axis is near saturation, with len/4 to mid-bucket the useful range. Treat v003's mid-insertion plus transpose as the policy baseline and try to cut its put cost, for example with a cheaper insertion mechanism that avoids shifting. Direct the next latency hypotheses at the lookup path (hashing, bucket search, dispatch) and validate on a second workload.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Variants targeted cost reduction around the mid-insertion/transpose baseline: a per-bucket ghost admission filter (v001), O(1) swap-based mid-insertion (v002), and single-hash/mask indexing with tag-first scan (v003). v002 was a clear Winner and v001 the other Winner on thin evidence, since its own insight reports a hit-rate loss and slower puts. v003 sped up put only.

## Patterns

- Put-path cost was the lever that moved. Removing the half-bucket shift (v002) and redundant hashing/modulo (v003) both cut put latency, with hit rate and metadata unchanged.
- Approximate ordering among displaced entries does not matter. Where the new entry lands matters, not the exact order of the rest, so exact bucket ordering is not worth paying for.
- A one-entry-per-bucket ghost is too weak a signal. It added put work and slightly hurt hit rate, penalizing reusable keys along with scan items.
- Get latency did not improve through any real mechanism. This is the fourth cycle with no get gain. Arithmetic, hashing and item movement are now all ruled out, and small get shifts look like noise or codegen effects.
- No variant grew metadata.

## Open questions

- Is the v001 Winner status real, or within noise given the negative insight?
- Is get time dominated by memory access to the bucket (cache lines, slot layout, tag/key separation)? It is untested.
- Would a deeper or more selective admission history help, or is scan resistance already saturated by insertion position?
- Do any gains hold on a workload other than workload-a?

## Continuation

Adopt v002's swap-based mid-insertion as the policy baseline and fold in v003's hash-once indexing only if it can be made to help rather than hurt get. Drop single-entry ghost filters and further arithmetic or hash micro-optimizations. Aim the next hypotheses at memory layout for get (cache-line alignment, packing tags or keys contiguously, prefetch) and validate on a second workload.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Variants probed memory layout for get (a contiguous tag-first scan, 64-byte bucket alignment) and nudged the insertion point from len/4 to len/3. None improved get. v001 and v002 were Winners, but both on thin evidence, since their own insights report a hit-rate loss and a slower put (v001), or a hit-rate loss with an unexplained put gain (v002).

## Patterns

- Get latency again did not move, now across five cycles. Arithmetic, hashing, item movement and layout (tags, alignment) are all ruled out. Buckets are probably small and cache-resident, so get is likely dominated by something outside bucket memory access.
- Layout changes that must stay in sync with promotions and swaps (parallel tags) raise write cost and risk drifting from the parent's policy. Pure allocation-placement changes (alignment) are safe, but their benefit showed up only on put tail latency.
- Insertion depth is a free hit-rate knob. Moving from len/4 toward the protected end lowered hit rate, which fits the earlier saturation finding that len/4 or lower is the useful range.
- Put-latency gains without a mechanistic reason (v002) are the same noise pattern seen in cycles 0001 and 0002.
- No variant changed metadata size.

## Open questions

- Are the v001 and v002 Winner statuses real, or are they artifacts of noisy put latency and a flat get?
- Does the optimum insertion point lie below len/4, such as len/5 or len/8?
- Where does get time go if not memory layout, hashing or reordering? It may be harness overhead, dispatch or the key and value path.
- Does cache-line alignment's put-tail gain repeat, and does it stack with swap-based insertion?
- Do any gains hold on a second workload?

## Continuation

Stop attempting get-latency gains through bucket layout, tags or hash arithmetic; the evidence says they do not work. Keep the swap-based len/4 insertion as the policy baseline, try slightly lower insertion points, and consider adopting plain 64-byte alignment as a cheap put-tail improvement after re-checking it. Add a second workload to validate any claimed Winner, and discount Winners whose own insight shows a loss.


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
