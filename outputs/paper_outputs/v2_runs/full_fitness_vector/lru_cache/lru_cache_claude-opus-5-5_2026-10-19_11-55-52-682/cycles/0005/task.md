# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0005

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
