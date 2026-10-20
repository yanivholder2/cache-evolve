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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants branched from cycle 0002's SIEVE and each tried one fix. v001 dropped the back-link (singly-linked queue, hand carries its predecessor). v002 replaced the visited bit with a 2-level counter. v003 capped the sweep length. All three are listed as Winners, but only v001 improved anything: a smaller node with the same eviction order and no speed change. v002 lost a little hit-rate and got slower. v003 lost a little hit-rate and did not move the put tail.

## Patterns

- Changing the representation while keeping the policy identical is the only safe way to gain on this lineage. It improved metadata without risk. Node compaction stopped helping per-op-time once the hit path no longer relinks, so on SIEVE it is a metadata lever only.
- Varying how the hand handles queued nodes has run out of value on workload-a. More credit per hit (v002) and less patience per sweep (v003) both cost hit-rate. SIEVE's single bit with quick demotion of new keys looks like a local optimum for in-queue reuse signals.
- This cycle refuted cycle 0002's diagnosis of the put tail. Capping the sweep did not lower put p99, so long sweeps are too rare to set it. The likely cost is the per-miss HashMap remove/insert, key clone and cold-node touches. GCLOCK's tail cost does reappear when credit is deepened (v002), but that is a separate issue from SIEVE's baseline tail.
- Long runs of visited nodes do occur at size 2048 and they carry real reuse. Evicting them early hurts.

## Open questions

- Would history of evicted keys (ghost entries, an S3-FIFO-style small/main/ghost split, or ARC/LIRS-style non-resident tracking) add hit-rate where in-queue counters could not?
- Can the per-miss map work be reduced, for example with a single entry-API remove-and-insert, by reusing the hash, or by avoiding the key clone? Would that lower put p99?
- Does SIEVE's lead hold at other sizes or traces? That has been unverified for two cycles.

## Continuation

Use v001 (singly-linked SIEVE) as the parent. Drop further counter-depth and sweep-cap tweaks to SIEVE. For hit-rate, try a signal from outside the queue: ghost/history of evicted keys, or an S3-FIFO-like small probationary FIFO with a ghost queue. For put p99, target the HashMap path on misses instead of the hand. Include a second cache size or trace this cycle before building further on workload-a at 2048.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants branched from singly-linked SIEVE (cycle 0003 v001). v001 grafted a ghost table of evicted keys onto the single queue. v002 swapped SipHash for an Fx-style hasher with `u32` map values. v003 moved the link and visited bit into a packed `u32` side array. All three are listed as Winners, but the outcomes split. v002 is the clear gain: faster get and put, hit-rate unchanged, and a smaller cache struct. v003 is a metadata-only gain with the same eviction order. v001 regressed on hit-rate and put latency.

## Patterns

- Representation-only changes stayed safe for the third cycle running. Both v002 and v003 kept eviction order bit-for-bit. The only policy change, v001, cost hit-rate.
- This cycle found the latency lever: hashing. Cycle 0003 already showed node layout no longer moves speed. v002 then moved get p50 for the first time in several cycles, and put p99 improved even more. This supports cycle 0003's view that the put tail comes from map work on the miss path, not from the hand. v003's denser sweep was roughly neutral on speed, which confirms that layout is now only a memory lever.
- A ghost signal needs a probationary queue to bypass. On a single SIEVE queue, a ghost hit can only extend the stay of keys whose reuse distance already exceeds the queue. Those keys push out better residents. v001 also added hashes on every miss, so the "outside-the-queue signal" question is still open. Only the minimal graft has been refuted.
- The metadata metric is drifting out of step with the real footprint. v001 put its growth in a heap table, and v003 put its policy state in a side array the per-node summary doesn't count. Read node-size wins with care unless side arrays are added in per entry.

## Open questions

- Does a real S3-FIFO split (small probationary FIFO + main SIEVE + ghost) gain hit-rate where the single-queue graft lost it?
- How much of put p99 remains after cutting probes per miss (entry-style insert, reusing one hash across the probe, remove and insert)?
- Do v002's hasher and v003's side array stack cleanly, and does the extra memory touch on each hit matter once hashing is cheap?
- SIEVE's lead at other cache sizes or traces is still unverified after three cycles.

## Continuation

Use v002 (Fx-hashed SIEVE) as the parent and fold in v003's side-array layout as a low-risk combination. For latency, cut the number of map probes on the miss path. Do not do further node-shape work for speed. Drop the single-queue ghost graft. If ghosts come back, they must come with a small probationary queue (S3-FIFO-style). Count side-array bytes per entry when judging metadata, and finally add a second cache size or trace.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Following cycle 0004, this cohort combined the Fx hasher with the side-array layout (v001), replaced hashbrown with a hand-rolled key-less `u32` index (v002), and added the first real S3-FIFO split: a small FIFO, a main SIEVE and a fingerprint ghost (v003). All three are listed as Winners. v003 is the real advance, with the first hit-rate gain in several cycles and no latency cost. v001 is a mixed combination: a better put tail and a smaller struct, but a flat get tail. v002 kept the eviction order but regressed on latency, sharply at the put tail.

## Patterns

- The open question about signals from outside the queue is now settled: ghosts help when there is a probationary queue to bypass. The same ghost idea that lost in cycle 0004 as a single-queue graft gained here inside an S3-FIFO split. The gain is modest, which suggests workload-a has only limited one-hit scan traffic.
- Structural policy changes can also fix the tail. S3-FIFO's O(1) small-queue evictions improved put p99 even though each miss did more hashing and ghost work. That is the first policy change in this lineage to help latency, and it suggests long main-hand sweeps still matter at the tail some of the time.
- The hashing lever has mostly been used up on the hit path. Cheap hashing now mainly helps the miss path and struct size. Get latency is now set by how many separate memory locations a hit touches, which is why the side array and Fx did not stack on get.
- Hand-rolled indexes lose to hashbrown when they cannot reject a bucket without dereferencing the arena. Key-less buckets and backward-shift deletion that re-hashes through the arena made the clusters much more expensive at the put tail. The idea of using one hash per miss is not wrong, but it is only worth doing inside a table that filters buckets cheaply.
- Metadata accounting is still incomplete. The S3-FIFO ghost heap and the denser index of v002 are both invisible to the current dimensions.

## Open questions

- How much hit-rate is left in the S3-FIFO knobs, such as the small-queue ratio, ghost size, and whether main evictions should feed the ghost?
- Can the visited bit sit next to the data a hit already reads, so the hit path touches one location instead of two without growing the node?
- Can hashbrown's entry or raw-table API deliver the single-probe miss path without giving up its tag filtering?
- Does S3-FIFO's gain hold at another cache size or trace? This has now been unverified for four cycles, and it matters more now that a policy change has finally won.

## Continuation

Use v003 (Fx-hashed S3-FIFO) as the parent. Tune its small-queue ratio and ghost sizing, and fold in v001's representation gains (key swap on eviction and the zero-sized hasher) as a low-risk combination. Drop hand-rolled key-less hash tables. For latency, use hashbrown's own entry or raw APIs, or place the visited bit with the data a hit already reads, rather than doing more hasher work. Count the ghost heap when judging metadata, and add a second cache size or trace before tuning S3-FIFO further on workload-a alone.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005's Fx-hashed S3-FIFO. v001 widened the small queue, v002 ported the packed `u32` side array with key swap, and v003 made the ghost 2-way associative and twice as large. All three are listed as Winners, but none advanced hit-rate. v002 is the only real gain: a smaller per-node footprint with the same eviction order, paid for with slower `get`. v001 and v003 both lost a little hit-rate and slowed `put`.

## Patterns

- On workload-a, S3-FIFO's admission knobs appear to be at or near a local optimum, as SIEVE's in-queue knobs were in cycle 0003. A longer probation window took space from main, and extra ghost recall pushed long-reuse keys into main. Both changes cost more than they recovered. Main capacity for proven-reuse keys is the scarce resource.
- The bounded reach of the direct-mapped ghost was acting as a useful filter. Collisions limited how far back a "returning" key could be and still skip probation. Ghost tuning should shorten reach, not lengthen it.
- The side array's latency effect depends on the policy. On SIEVE it helped the `put` tail. On S3-FIFO's heavier miss path, that benefit disappears, and `get` now touches two cache lines (flags and value) instead of one. Representation changes still preserve eviction order safely, but they are no longer free on latency.
- The `put` path on S3-FIFO is now dominated by hashing and ghost probes on each miss. Changes that add ghost work show up at the `put` tail right away.
- Metadata accounting is still blind to heap side structures: the larger ghost and the `links` buffer and `Vec` header are not counted. The per-node wins overstate the real savings.

## Open questions

- Can VISITED stay inline with the value while only the links move out, keeping most of the memory win without the `get` penalty?
- Would a smaller ghost, sized to the small queue or to main's reuse distance, or a softer admission for ghost hits (e.g. entering main unvisited behind the hand) beat the cycle-0005 configuration?
- Does any of this hold at another cache size or trace? This has been unverified for five cycles, and the lineage is now tuning against a single point.

## Continuation

Keep cycle 0005 v003 as the hit-rate parent. Use cycle 0006 v002 only where memory matters, and try a variant that keeps VISITED next to the value. Drop wider small queues and larger or more associative ghosts. If ghost tuning continues, go smaller or make ghost admission less aggressive, and test split points just around 10% (1/8, 1/12) rather than wider. Before any further knob tuning, evaluate at a second cache size or trace, and count ghost and side-array heap bytes when judging metadata.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle tried one representation change and two ghost-policy changes. v001 moved VISITED from cycle 0006 v002's `links` word into a dense bitset. v002 placed ghost returners at the SIEVE hand. v003 halved the direct-mapped ghost. All three are listed as Winners, but only v001 is a clear win: `get` and the `put` tail are faster and eviction order is unchanged. v003 is a near-noise hit-rate gain that comes with a free heap saving. v002 lost hit-rate.

## Patterns

- Both ghost probes this cycle point the same way: limit how far back the ghost reaches, but don't weaken what happens after a ghost hit. Returners placed at the hand were evicted before they could reuse. Shortening the ghost's reach cost nothing and may have helped slightly. A ghost hit is a strong reuse signal. Recall is the knob worth tuning, not admission placement.
- Flags that get written on every hit belong in a tiny, always-cached structure. The bitset recovered most of the side array's `get` penalty and made hand sweeps cheaper at the `put` tail. Each extra side array still adds fixed work on the miss path and grows the struct, so the median `put` rose slightly.
- S3-FIFO knobs on workload-a are now a flat plateau. A fourth straight cycle of admission and ghost tweaks produced only noise-sized moves in either direction.
- Latency attribution is getting unreliable. v002 slowed on a `get` path its change never touched, so environment drift is now as large as the effects being measured.
- Metadata accounting still misses heap side structures. v003's real ghost-buffer saving and v001's bitset don't show up in it.

## Open questions

- Should main evictions be recorded in the ghost? v002 found that returners evicted from main lose their fast path.
- Where does hit-rate start to fall as the ghost shrinks further, and does that point move with cache size?
- Can a single fused node-plus-flag layout give the bitset's `get` gain without an extra array header or extra work on every miss?
- Does any of the S3-FIFO lineage hold at another cache size or trace? This has gone unverified for six cycles.

## Continuation

Use v001's bitset layout combined with v003's half-size ghost as the new parent. Both keep eviction order safe, and they should stack. Drop placing returners at the hand, and stop sweeping small-queue or ghost size on workload-a. Before more policy tuning, add a second cache size or trace and repeat runs so latency deltas can be told apart from drift. The one policy probe still worth trying is recording main evictions in the ghost.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007 v001 (the VISITED bitset). v001 stacked the half-size ghost on top, v002 recorded main evictions in the ghost, and v003 moved VISITED into a spare bit of the map value. All three are listed as Winners. The two policy changes each gave a small hit-rate gain with no real memory or latency cost. The representation change won on the medians and on struct size but lost badly at the `put` tail.

## Patterns

- The half-size ghost gain held on a second node layout, so it is a real policy gain that does not depend on layout. Main-eviction ghosting was also positive, but barely. Together they close out ghost tuning on workload-a: what the ghost reaches back to and what feeds it now matter only at the margin.
- Where a flag lives has to match how it is accessed. The hit path reaches entries by key, but the SIEVE hand and small-queue pops reach them by index. Putting VISITED in the map bucket helped the readers that come in by key and made every hand step a hash plus a scattered probe. The `put` tail is set by the eviction scan, so index-addressed flag storage (the bitset) is still the better overall trade.
- The hit path is close to its floor. With the third structure gone, the gain was sub-nanosecond: one map probe plus one value line now accounts for nearly all of `get`.
- Latency measurement is still unreliable. v001's code left the `get` path untouched, yet its latencies all slowed by about the same proportion. Cross-run latency deltas cannot currently be attributed to a change.

## Open questions

- Do the half ghost and main-eviction ghosting stack, or do they interfere? A smaller table makes dilution from main victims worse.
- Can the bucket-flag `get` win and the bitset's cheap hand both be kept, for example by mirroring the flag or by having the hand sweep touch fewer nodes?
- Does any of the S3-FIFO lineage hold at another cache size or trace? This has gone unverified for seven cycles.

## Continuation

Use cycle 0008 v001 (bitset plus half ghost) as the parent. Optionally fold in v002's main-eviction ghosting as a single combined check, then stop probing the ghost. Drop map-bucket VISITED unless the hand gets index-addressed access back. Before more micro-tuning, put latency comparisons in the same run as the parent and evaluate at a second cache size or trace. Further hit-rate gains will probably need a structurally different idea, not more S3-FIFO knob changes.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants branched from cycle 0008 v002, which ghosts main evictions. v001 halved the ghost on top of it. v002 added a TinyLFU count-min duel at small-queue exit. v003 moved values into the map buckets. All three are listed as Winners, but only v001 is non-negative: a sliver of hit-rate gain with no cost. v002 lost on every axis. v003 left hit-rate unchanged and slowed both `get` and `put`.

## Patterns

- Ghost tuning is finished. The half ghost and main-eviction ghosting interfere rather than stack. Both make the ghost more selective, so combining them mostly adds collision dilution. The hypothesis's own exit condition is effectively met.
- The first structurally new admission signal, frequency, also failed to break the plateau. It reduced hit-rate because the duel ran the SIEVE hand speculatively and wore down main's VISITED protection. The sketch also undercounted hot main keys while overcounting churny probationary ones. The ghost already captures most of the returning-key signal on workload-a. This is more evidence that the plateau belongs to the trace, not to the admission policy.
- The hit path's floor is the map probe, not the value line. At this capacity the 16-byte arena stays cache-resident, so removing the `nodes[idx]` read saved nothing. Widening buckets made every probe, insert and remove slower. Map bucket width is now the latency lever, and it is better made smaller than larger.
- Metadata accounting is still blind to the heap, which made all three results look better than they are: the ghost table, the sketch, and the larger hash table all go uncounted. v003's apparent per-node halving only moved bytes into the map.

## Open questions

- Would frequency admission work if it only peeked at the victim, without clearing VISITED bits, and counted main hits on every hit? Or is workload-a simply unable to reward any admission signal?
- Can the map bucket shrink below key+`u32`, for example with a fingerprint-indexed or open-addressed table keyed into the arena?
- Does the S3-FIFO lineage hold at another cache size or trace? This has gone unverified for eight cycles, and it is now the main threat to every conclusion above.

## Continuation

Make cycle 0009 v001 the parent, and treat ghost size and inputs as closed. Drop value-in-bucket layouts and drop the TinyLFU duel as specified. Revisit frequency only with a non-destructive victim peek and faithful hit counting, and only after it is shown to matter on a second trace or cache size. That evaluation should be the next cycle's priority over any further workload-a tuning. For latency, target narrower or cheaper map probes.


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
