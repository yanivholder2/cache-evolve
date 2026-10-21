# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from the baseline, each along a different axis: resident-list structure (W-TinyLFU window + SLRU, v001), sketch freshness via periodic aging (v002), and hashing/sketch layout (v003). The winner, v003, was a pure efficiency change. It lowered get and put latency at both p50 and p99 and shrank the cache struct slightly, while leaving hit rate unchanged.

## Patterns

- Hit rate did not move in this cycle. Both completed variants kept the baseline `CountMinSketch` counting rules, and both stayed within noise on hit rate. That suggests the admission signal, not the list structure, is what caps hit rate right now.
- Per-op cost in the baseline came mostly from hashing, not from cache logic. Removing the repeated per-row SipHash calls brought large latency gains, and the put tail gained most because it runs the admission contest.
- Adding structure has a cost that only pays off if hit rate rises with it. v001's extra region tags, per-list arrays and per-hit relinking made memory and latency worse and gave almost no hit-rate gain.
- v002, the aging hypothesis, never ran. It was rejected because of an infrastructure failure (an HTTP 429 from Maven while downloading the Caffeine simulator), not because of anything in the hypothesis. Its idea is still untested.

## Open questions

- Does sketch aging/reset (with a lower saturation cap) actually raise hit rate on workload-a, and does it make a window/SLRU structure worthwhile once the frequency signal stays fresh?
- Is the unkeyed fast hasher an acceptable risk if a trace has adversarial or collision-prone keys?
- Would a larger window than about 1% change v001's outcome, or does the window size matter little while the sketch never ages?

## Continuation

Build on v003, since it is now the cheap-hashing baseline. Re-submit the H02 sketch-aging idea on top of it, because the infra failure means it was never evaluated, and target hit rate through the frequency signal. Drop further resident-list restructuring such as SLRU or window variants until aging has been shown to move hit rate. If a structure is revisited later, it must keep the per-hit relink cost low.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants branched from cycle 0001's v003, the fast-hash version. v001 added Caffeine-style sketch aging with a 4-bit cap. v002 packed the list links into compact `u32` indices. v003 picked the victim as the coldest of a small sample of LRU-end entries. Only v002 produced a real gain, a halved per-entry `Node` size. No variant raised hit rate, and all three regressed on per-op time at least partly.

## Patterns

- Hit rate is still flat or worse. Aging (v001) and frequency-aware victim choice (v003) both tried to use the sketch more, and neither helped. Aging alone made hit rate worse: under the strict `>` admission rule with ties going to the victim, coarser counts mean more ties, so more good newcomers get rejected. Victim sampling changed nothing, because the entries near the LRU end have nearly equal estimates.
- The admission rule and the counter resolution interact. Changes to the frequency signal can't be judged on their own while the tie-break rule and the lack of a window or doorkeeper stay fixed.
- Any extra work in the hot path shows up in latency right away. Per-op data-dependent branches and bookkeeping (v001), and extra per-miss hashing (v003), each gave back latency that cycle 0001 had gained.
- Layout-only changes are the one reliable win. v002 shrank the metadata with no change in behaviour. Its latency regression has no credible cause in the code and looks like benchmark-environment noise.

## Open questions

- Is v002's per-op regression real? Benchmarking it side by side with cycle 0001's v003 on the same machine would settle this.
- Would aging help if paired with a tie-aware or randomised admission rule, or with a small admission window or doorkeeper, and with cheaper reset bookkeeping?
- Is workload-a's hit rate close to its ceiling for a TinyLFU-family design, so that efficiency is the only axis left to improve?

## Continuation

Make v002 (compact links) the new base, and first re-measure its latency against cycle 0001's v003. Drop LRU-end victim sampling. Don't re-submit aging on its own. If you revisit aging, pair it with a change to admission or tie handling and keep the bookkeeping out of the per-op hot path. Layout or efficiency changes that add no hot-path work remain the safest bets.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants branched from cycle 0002's v002 (compact `u32` links). Each made one isolated change to admission or the sketch: v001 let newcomers win ties (`>=`), v002 put all of a key's counters into one cache-line block, and v003 switched to conservative update. Only v003 raised hit rate, slightly. That is the first hit-rate gain in the run. v001 and v002 lowered hit rate a little.

## Patterns

- The hit-rate ceiling comes from how accurate the estimates are, not from the tie rule. Making ties favour the newcomer hurt, so on workload-a the strict `>` rule was not over-rejecting. Reducing count-min overestimation helped, because it keeps cold newcomers and cold victims further apart in the admission contest.
- Anything that adds correlated collisions hurts admission. The blocked layout broke row independence and inflated estimates, which is the opposite of what conservative update does. Sketch accuracy is the lever, and it can move either way.
- The sketch already fits in cache at these capacities. Layout changes aimed at cache misses have nothing left to win, so the earlier "layout-only is safe" heuristic does not cover the sketch.
- Per-op-time readings stayed unreliable. v001 was faster on every op with no credible cause in the code, which matches the environmental drift suspected last cycle. Small latency differences between variants still can't be trusted.
- v003's gain cost some typical `get` latency from its two-pass increment. Only `put` gets that cost back, by reusing the returned estimate.

## Open questions

- Can the conservative-update increment be made single-pass, or kept in registers, so that `get` loses its extra cost?
- Now that conservative update limits the build-up of overestimates, would aging or a doorkeeper help where aging alone failed in cycle 0002?
- Would a wider or deeper sketch, with row independence kept, extend v003's accuracy gain for an acceptable metadata cost?

## Continuation

Use v003 (conservative update) as the new base, and first tighten its increment loop to recover `get` latency. Keep pursuing estimate accuracy: lower-collision sketch sizing, or a doorkeeper or aging paired with conservative update. Drop the tie-admission changes and the blocked or cache-line sketch layouts. Benchmark any latency claim side by side against the parent.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants branched from cycle 0003's v003 (conservative update), and each made one change to the sketch. v001 fused the increment into a single pass held in registers. v002 added Caffeine-style halving every 10×capacity increments. v003 swapped to `u16` counters at twice the width while keeping the sketch the same size in bytes. All three are listed as Winners, but they won for different reasons. v001 recovered per-op cost and left hit rate unchanged. v003 raised hit rate slightly at no metadata cost. v002 lowered hit rate, and its faster per-op times are most likely environmental noise, not a real gain.

## Patterns

- How conservative update is written now costs almost nothing. v001 showed that the earlier `get` penalty came from the two-pass reload loop, not from the update rule itself. What's left is within the drift of the bench.
- Sketch accuracy is giving less and less. Halving collision noise (v003) raised hit rate only a little. Once conservative update is in place, CountMin overestimation is close to used up as a lever.
- Decay hurts on workload-a no matter what counter it's paired with. Aging failed with 4-bit counters in cycle 0002 and failed again here with full-width counters and conservative update. Workload-a rewards stable long-term frequency. Under a strict duel against the LRU head with no window, halving removes the margin that keeps scan traffic out.
- Saturating `u16` counters cost nothing, so wider counters were wasted bytes. The bytes saved this way are better spent on width.
- Per-op readings are still unreliable. Two variants with no credible cause for a speedup (v002, v003) both got faster on every op. Only v001's latency gain has a mechanism in the code that explains it.

## Open questions

- Do v001's fused increment and v003's `u16`/2× width combine cleanly, giving both the latency and the hit-rate gain at once?
- Now that sketch accuracy has stopped giving gains, is the remaining hit-rate gap in the eviction or admission policy: the plain-LRU victim, the lack of an admission window or doorkeeper?
- Could aging work in a much milder form, with a far longer period or a sample that counts only misses rather than hits on resident keys? Or should it be dropped for this workload?

## Continuation

Merge v001's fused increment with v003's `u16`/2× width sketch as the next base, and confirm that both gains hold side by side against each parent. Drop blanket periodic halving, and stop tuning sketch width or overestimation. Look for hit-rate gains in policy structure instead, such as a small admission window or doorkeeper ahead of the duel. Keep per-hit and per-op cost low, because cycle 0001 showed that heavy list restructuring doesn't pay off.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Two variants branched from cycle 0004's v003 (`u16`/2× sketch). v001 merged in the fused increment, and v002 removed the double count on get-miss followed by put. v003 branched from cycle 0004's v001 and added a ~1% plain-LRU admission window, with the region flag packed into a link bit. v002 gave the only hit-rate gain, and it was small. v001 merged cleanly but carried over none of the latency gain. v003 lowered hit rate and raised per-op cost. All three are listed as Winners, but only v002 moved a headline axis forward.

## Patterns

- The fused-increment latency gain did not carry over to the `u16` sketch. Hit rate was identical bit for bit, so the merge is behaviourally sound. That makes cycle 0004/v001's speedup look like either a 32-bit-counter effect or bench drift. Sketch micro-optimisation is used up, just as sketch accuracy was last cycle.
- How accesses are counted barely matters. The +2 bias on a miss applied to residents and newcomers alike, so it mostly cancelled in the duel. Fixing it only changes close duels. Together with cycle 0003's tie-rule result, this suggests the duel's inputs are now about as fair as they can get.
- A tiny window ahead of the duel hurts on workload-a. At ~1% it is too small to hold recency bursts. It also moves the duel from the arriving key to an older window head, which reorders admission badly. Every hit then pays for list selection and masked links, and every full miss re-hashes stored keys. This matches cycle 0002's aging result: the workload rewards stable frequency, and recency-oriented structure doesn't pay for itself.
- Packing the region flag into a link bit costs nothing per entry. If structural changes come back later, per-node bytes are no longer the obstacle.
- Per-op readings are still unreliable. v002 slowed down with no plausible cause in the code. Only v003's regression has a clear mechanism behind it.

## Open questions

- Would a larger window, around 5–20%, with cached per-node hashes or estimates so the overflow duel doesn't re-hash, behave differently? Or is any window wrong for this trace?
- With sketch and admission inputs both used up, is the remaining cost in `HashMap` lookup, `key_hash`, and relinking on every hit, rather than in the sketch?
- Is workload-a's hit rate now close to its ceiling for this design family?

## Continuation

Use v002 (single count per access) as the base. Optionally fold in v001's merged increment: it is orthogonal and leaves hit rate unchanged. Re-measure its latency side by side against cycle 0004's v003 before treating the regression as real. Drop sketch micro-optimisation, access-accounting tweaks, and tiny (~1%) windows. If you test a window again, make it substantially larger and cache each node's hash so the duel does no re-hashing. Otherwise, move the effort to the costs outside the sketch: map lookup, hashing, and relinking on every hit.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort moved from the sketch to the structures around it. v001 replaced the LRU list with CLOCK reference bits. v002 replaced the std map with a custom index table that stores only node indices. v003 added a 10% admission window and folded in the fused sketch increment. All three are listed as Winners, but only v001 is a clean step forward: smaller per-entry metadata with hit rate unchanged. v003's latency gain comes from the fused increment, not the window. v002 brought no gain.

## Patterns

- Admission doesn't depend on exact recency order. CLOCK's coarse second-chance order left hit rate flat, so exact LRU position was never giving the duel any useful information. Per-node links can go without losing quality.
- Windows are now ruled out at both sizes. The ~1% window (cycle 0005) and the 10% window (this cycle) both lowered hit rate. Workload-a rewards strict frequency filtering at the door. Any capacity given to unfiltered arrivals goes mostly to one-hit keys.
- Median `get` cost doesn't sit in relinking or in hashing twice. Removing relinks (v001) and removing the duplicate hash (v002) both left the `get` median unchanged. The only change that moved latency was the fused register-held sketch increment (v003). That suggests cycle 0005's failure to carry this gain over was bench drift, not a real regression.
- Tail costs show up on the rare paths. CLOCK's unbounded sweep, and v002's second probe plus re-hashing during backward-shift deletion, both made `put` p99 worse. Structural changes need to keep eviction-path work bounded and free of extra hashing.
- A custom index that stores only node indices does worse than SwissTable. Every probe then has to load from `nodes`, which costs more than the hash it saves.

## Open questions

- Does word-at-a-time sweeping (`trailing_ones` over a `u64` of ref bits) bring CLOCK's `put` tail back to LRU's O(1) level?
- Would an index that stores hash tags or home slots beat std `HashMap`, or is the map simply not the bottleneck?
- With recency structure, sketch accuracy and duel inputs all used up, is workload-a's hit rate at its ceiling for this design family?

## Continuation

Use v001 (CLOCK) as the new base. Merge in v003's fused increment without the window, and bound the sweep with word-level bit scanning. Confirm the latency side by side against both parents. Drop admission windows of any size, and drop index-only custom hash tables. Treat further hit-rate work as low-yield. Prefer metadata and tail-latency reductions that keep the duel and the hit rate unchanged bit for bit.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants branched from cycle 0006's v001 (CLOCK). v001 replaced the per-bit sweep with a word-at-a-time scan. v002 collapsed the fill-phase double map probe into one `entry` call. v003 let the duel compare the newcomer against the colder of two clear-bit CLOCK candidates. All three are listed as Winners, but none moved a headline axis forward. v001 and v002 kept hit rate identical and made latency slightly worse. v003 lowered hit rate.

## Patterns

- Both latency fixes optimised paths the trace rarely takes. Long CLOCK sweeps are rare, because newcomers enter with a clear bit. The fill phase runs only once, during warm-up. Before targeting a path, measure how often steady state actually reaches it.
- Fixed per-call overhead and struct layout matter more than worst-case work. Even a small fixed cost, such as mask math, an out-of-line call or one extra struct field, can outweigh a rarely used saving. Growing `TinyLfuCache` past two cache lines may be what slowed down an unchanged `get`.
- `get` latency moved on all three variants, even though no variant changed `get`. The p99 tails in particular can't separate small edits from noise or layout shifts. A gain only counts if a mechanism explains it and it repeats against the parent side by side.
- Using more candidates to choose the victim hurt again, as cycle 0002's LRU-tail sampling did. Taking the minimum of two estimates loosens admission. Leaving the hand parked next to the slot just filled makes new admits the next victims, so they evict each other. The strict single-victim duel at the door is still the best admission rule for workload-a.
- Both behaviour-preserving variants kept hit rate exactly, so the CLOCK base is stable to refactor.

## Open questions

- Does `put` p99 for the CLOCK base come from the sweep at all, or from the full-miss map work: a `get` probe on rejection, then `remove` and `insert` on admission, each re-hashing the key?
- Would a hash-aware/raw-entry API that reuses the precomputed `h` across those three map operations cut steady-state `put` cost?
- Would a two-candidate scheme help if the admission threshold stayed tied to `a` and the hand moved past both candidates? Or should this line be closed?

## Continuation

Keep cycle 0006's v001 (CLOCK) as the base, and don't adopt any of this cycle's variants. Drop sweep micro-optimisation, fill-phase map tweaks and min-of-candidates admission. Don't add struct fields to the cache's hot layout. If latency is the target, aim at the steady-state full-miss map path, for example by reusing the precomputed hash across get/remove/insert. Count how often each path runs before targeting it, and judge success by medians repeated against the parent side by side, not by single p99 readings.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Each variant was on the CLOCK line and tested one lever. v001 keyed the map by the precomputed hash so each key is hashed once. v002 doubled the sketch width again. v003 put a periodically cleared doorkeeper in front of the sketch. All three are listed as Winners, but none moved a headline axis forward. v001 kept hit rate identical but slowed down a little. v002 gained a hit-rate amount no bigger than noise and paid for it in median latency. v003 lowered hit rate and raised latency.

## Patterns

- Hashing is not the cost. Cycle 0006 removed a duplicate hash and this cycle removed the map-side re-hashes, and neither changed latency. For `u64` keys the Fx-like hash is nearly free, and any guard compare or extra branch on the hit path costs more than it saves. Drop hash-count reduction as a latency lever.
- The sketch's memory footprint is now the clearest per-op cost. This is the first time a median regression has a clear mechanism behind it: the doubled table no longer sits comfortably in L1/L2, so `get` p50 went up even though `get` code was untouched. Each access makes four scattered counter writes, and that is where the `get` time goes.
- Sketch-input levers are used up from both directions. More width buys nothing measurable. Filtering first sightings (the doorkeeper) hurt, because a small filter fills with one-hit keys and its all-at-once clear acts as abrupt aging. That matches the earlier halving and aging failures. The strict frequency duel with non-decaying counters still wins on workload-a.
- Refactors that preserve behaviour still keep hit rate exactly (v001), so the CLOCK base stays safe to change. The p99 readings still move in both directions with no explanation in the diffs.

## Open questions

- Would a smaller or denser sketch (narrower counters, rows packed into one cache line per key, or fewer rows) cut `get` latency without losing hit rate? This is the reverse of this cycle's widening.
- Is workload-a's hit rate at its ceiling for strict-duel TinyLFU on CLOCK? Every admission, aging, window and sketch-accuracy change since the early cycles has been flat or negative.

## Continuation

Keep cycle 0006's v001 (CLOCK, width factor 8) as the base, and adopt none of this cycle's variants. Drop the following:
- hash-once and identity-keyed maps
- further sketch widening
- doorkeepers and any periodic or abrupt aging

If latency is the target, cut the sketch's memory traffic on each access (for example, rows blocked into one cache line, or a smaller footprint). Require hit rate to stay identical or within noise, and judge by medians repeated side by side against the parent. Otherwise, treat hit rate as near its ceiling and prefer behaviour-preserving changes.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants went after cycle 0008's claim that the sketch's per-access memory traffic was the main `get` cost. v001 narrowed counters from `u16` to `u8`, v002 software-prefetched the counter lines ahead of the map probe, and v003 put each key's four counters into one 64-byte block. All three are listed as Winners, but only v001 is a real step forward. It halves sketch memory, trims `put` p50 and both tails, and keeps hit rate unchanged. v002 and v003 both raised median latency, and v003 also lowered hit rate.

## Patterns

- Cycle 0008's memory-traffic diagnosis was mostly wrong at this size. The sketch stays in L1/L2 under the benchmark. Halving it left `get` p50 flat, and the two changes built to hide or remove misses only added ALU work to the critical path: extra index math and prefetches in v002, an extra `fmix64` in v003. Making the sketch smaller helps a little. Hiding latency or relaying out the table does not.
- Any fixed cost on the per-access sketch path shows up at the median. A second `indices()`, an extra hash mix or extra prefetch instructions each cost more than the miss savings they targeted. Cycle 0007's lesson about fixed overhead beating rare-path savings applies here too.
- Counter range was wasted, but the independence of counters across rows was not. Saturating at 255 never changed a duel, which again shows admission is decided by low counts. Blocking counters together, even with disjoint lanes, ties the rows to each other and inflates cold keys' estimates. That is the second time a blocked layout has failed, after cycle 0003.
- `put` gains more than `get` from sketch shrinkage, because a full-miss `put` walks the sketch twice. `get` p50 has now resisted changes to relinking, hashing, sketch size and sketch layout.

## Open questions

- Would packed 4-bit counters (saturation at 15) keep the duel intact and shrink metadata further, or would the nibble masking cost the median more than the smaller footprint saves?
- Is `get` p50 at its floor for a SwissTable lookup plus one sketch increment, given that hash-count reductions also failed to move it?

## Continuation

Adopt v001 (CLOCK with saturating `u8` counters) as the new base. Drop software prefetching and all blocked or one-line sketch layouts. Don't add hash mixes or index recomputation to the sketch path. A 4-bit counter variant is the one sketch lever still worth trying: require the hit rate to stay exactly the same and judge it by medians repeated side by side against v001. Otherwise, treat `get` p50 and hit rate as near their ceilings, and go for metadata reductions that preserve behaviour.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

All three variants branched from cycle 0009's v001 (CLOCK with `u8` counters), and each tested a separate lever. v001 packed the sketch into 4-bit counters. v002 replaced the 1-bit CLOCK reference bit with a 2-bit GCLOCK counter. v003 replaced the `fmix64` finaliser with a single xor-fold. All three are listed as Winners, but only two are real steps forward. v003 is the first change to lower `get` p50, and hit rate stays within noise. v002 is the first eviction-order change to raise hit rate, and it pays for it in tail latency. v001 lost hit rate and latency together.

## Patterns

- The `get` median is limited by the serial ALU chain before the counter loads, not by memory. Removing hash-finaliser work helped. Shrinking the sketch (v001 here, and cycle 0009) or adding mix and prefetch steps (cycle 0009) did not. This sharpens cycle 0008's claim that "hashing is not the cost": the number of hash calls doesn't matter, but the latency of the finaliser on the sketch path does.
- Without aging, counter range is not spare. Counts only grow, so a cap of 15 makes hot residents and hot newcomers tie at saturation, and the strict `>` rule rejects the newcomer. The `u8` cap of 255 was fine. The lesson from cycle 0009 that admission is decided by low counts holds only when the cap sits far above the counts that steady state reaches.
- Hit rate moved when eviction order inside the main region changed, while the admission duel stayed fixed. Every admission-side lever has been flat or negative. Giving residents with repeat hits more lives changes which cold resident the duel defends, and that helped.
- Extra fixed per-access work still shows up on the hot path, as in cycles 0007 and 0009. Nibble shift and mask math (v001) and a second bit-plane RMW plus longer sweeps (v002) both cost tail latency. v003 removed work and was the only variant whose median fell.
- `put` p50 and the `get` p99 tail did not respond to any lever this cycle. `put` is limited by map remove/insert and the sweep.

## Open questions

- Do v002 and v003 compose? Hash folding should be independent of GCLOCK victim choice.
- Would interleaving the two GCLOCK bits per slot in one word remove v002's extra RMW, and with it the tail regression, while keeping the same victim choice?
- Does a 3-bit usage counter add more hit rate, or does the longer worst-case sweep eat the gain?

## Continuation

Use v003 (xor-fold hash) as the new base. Then port v002's GCLOCK onto it, with both usage bits interleaved in a single word per slot, and require the hit-rate gain to survive with tails no worse than v003's. Drop 4-bit counters unless they come with halving or aging, and drop sketch-footprint changes as a latency lever entirely. Don't trim the hash further, because the remaining `put` and tail costs are in the map operations and the sweep.


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
