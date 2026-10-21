# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants came from cycle 3's v001 (ghost at half of capacity) and each tried a different way to filter the ghost-hit path into `main`:
- **v001** shrank the ghost again, to a quarter of capacity. It won with a modest gain.
- **v003** kept the ghost size and added a count-min frequency gate on ghost re-admission. Its gain was only marginal.
- **v002** routed ghost hits into `small` and required two hits for cold keys. Its hit rate regressed.

## Patterns

- **Shrinking the ghost is still the best lever, but the gains are getting smaller.** Two halvings in a row have helped, and this one helped less than the last. The optimum is probably close to a quarter of capacity or a little below it. It has only been measured at one cache size.
- **A ghost hit is a useful reuse signal once the window is short.** Cycle 2 called ghost hits misleading on long loops. With a short window, though, they mostly catch real reuse. v002 made ghost-hit keys prove reuse again inside the tiny probation queue and lost hit rate. v003's frequency gate found little left to filter.
- **Requiring reuse while resident has hit a limit.** In earlier cycles, stricter admission kept paying off. Here it stopped: with a probation queue of about 10%, a two-hit bar or a forced pass through `small` evicts keys whose reuse gap is only slightly longer than that queue. Admission can be too strict as well as too loose.
- **Gates on the ghost path leak.** Both v002 and v003 left keys a second route into `main` through `small`, the same leak cycle 3 found with the promotion bar. v003 also compared against `main.front()`, which is often not the entry that actually gets evicted. Any admission gate should compare against the real eviction victim.
- **Measurement is still missing, for the fourth cycle running.** v001's smaller ghost probably saves memory. v003's sketch probably costs both memory and time, since it adds a table and extra hashing on every operation. Neither effect has been verified.

## Open questions

- Where does hit rate turn over: an eighth of capacity, or lower? Does the best ghost size hold at other cache sizes?
- Was v002's regression caused by the routing change or by the two-hit bar? The two were never tested separately. Its ghost bookkeeping fix, which stops stale entries erasing newer ones, is correct and should be kept.
- Would TinyLFU-style admission against the real eviction victim, applied to all insertions and not only ghost hits, pay back the sketch's unmeasured cost?

## Continuation

Use this cycle's v001 as the parent. Run one more ghost-size probe at an eighth of capacity, at more than one cache size if possible, then freeze the ghost size. Port v002's sequence-number ghost fix by itself as a correctness change.

Drop forced routing of ghost hits through `small`, and drop two-hit promotion bars for now. Drop ghost-path-only frequency gates. Bring the sketch back only as victim-based TinyLFU admission, and only once `per-op-time` and `metadata-size` can be measured. With the ghost size settling, run the long-postponed representation-only experiments next: the fingerprint ring and the hasher plus narrow-index graft. Getting the harness to measure those two dimensions is still the top priority.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants came from cycle 4's v001 (ghost at a quarter of capacity). Each ran one of the experiments cycle 4 asked for:
- **v001** halved the ghost again, to an eighth of capacity. It won, with a smaller gain than the previous halving.
- **v002** swapped the ghost to a fingerprint ring at the same window size, changing only the representation. Hit rate was exactly unchanged.
- **v003** added TinyLFU admission on every route into `main`, compared against the real CLOCK victim. Its gain was only marginal.

## Patterns

- **Shrinking the ghost still pays, but by less each time.** Three halvings in a row have helped, each less than the one before. The ghost window is now about the size of the probation queue, so the next halving would make the ghost smaller than `small`. That probe really asks whether the ghost-into-`main` fast path is worth having at all. Every result so far comes from one trace at one cache size.
- **Frequency admission is near its ceiling on this design.** v003 fixed both flaws found in cycle 4: the leak through `small` and comparing against the wrong entry. It still gained only a hair. The small-queue filter plus a short ghost already turns away most of the low-frequency candidates that TinyLFU is meant to block. So the leak and the wrong target were not what held the sketch back.
- **The representation track finally ran, and as expected it left hit rate alone.** v002 confirms the fingerprint ring is a safe base for later ghost work. The stale-erase fix made no measurable difference, because the case it guards against almost never happens. Whether the ring is a net win isn't known yet: it removes key clones but adds an extra hash pass on each miss.
- **Measurement is still missing, for the fifth cycle running.** This cycle makes the gap costly. One variant's only claimed benefit sits entirely in the unmeasured dimensions. Another adds a per-access sketch whose memory and time cost could outweigh its tiny hit-rate gain. Neither can be judged.

## Open questions

- At a sixteenth of capacity (ghost smaller than `small`), does hit rate finally turn over, or does the ghost path turn out to be dispensable?
- Do the ghost-size gains hold at a second cache size or on another trace, or do they just fit workload-a's reuse distances?
- Does the fingerprint ring actually cut `metadata-size` and `per-op-time`, given its extra hashing? Is the sketch's per-access cost close to zero?

## Continuation

Use this cycle's v001 as the parent. Run the sixteenth-of-capacity probe, and if at all possible run it at a second cache size as well, before trusting further tiny ghost-size gains. Port v002's fingerprint ring onto that parent as a policy-neutral base. Keep its hashing cost in mind: one hash per key, used for both the fingerprint and the lookup, if that can be arranged.

Drop TinyLFU and other frequency-admission gates unless cost measurements show they are nearly free. Look for hit-rate gains outside admission filtering. Measuring `metadata-size` and `per-op-time` is now blocking: without it, the representation and sketch results can't be judged.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants came from cycle 5's v001 (ghost at an eighth of capacity, `small` at a tenth). Each changed one knob:
- **v001** halved the ghost to a sixteenth of capacity. Hit rate was flat.
- **v002** widened `main`'s frequency cap from 3 to 7. It gained a little.
- **v003** halved `small` to a twentieth of capacity and kept the ghost at an eighth. It won.

All the gains were small.

## Patterns

- **Ghost and `small` act as one admission window.** v001 and v003 point the same way. With `small` at a tenth of capacity, a ghost smaller than `small` adds almost nothing. Once `small` shrinks, a ghost larger than `small` does real work as a cheap second stage of probation. It costs a ghost key, not a resident slot, and the freed slots go to `main`. The question is no longer "how small can the ghost be". It is "how should the combined window be split between resident probation and ghost".
- **Ghost-only tuning is done.** The ghost-size curve has gone flat. Taken alone, v001 suggests the fast path is dispensable. v003 shows that conclusion holds only while `small` stays large.
- **Retention is a separate lever, and it barely moves.** v002 is the first winner-grade result outside admission. Its gain is about as small as the last ghost halving. Telling moderately hot keys apart from very hot keys matters little on this trace. It is orthogonal to v003's change, though, so the two could stack.
- **The evidence base is getting thin.** Every gain this cycle is small, and all of them come from one trace at one cache size. Results at a second size were asked for in cycles 4 and 5 and still haven't come. v003's in-file tests don't exercise the new ratio at their tiny capacities. The fingerprint-ring port that cycle 5 asked for was not attempted. `metadata-size` and `per-op-time` are unmeasured for the sixth cycle running.

## Open questions

- With `small` at a twentieth, what ghost size is best? Is the right rule to keep `small + ghost` roughly constant while moving slots from `small` to the ghost?
- How far can `small` shrink before keys that arrive in `main` with no frequency credit start to hurt?
- Does v002's wider cap stack with v003's split, or do the two overlap?
- Does any of this hold at a second cache size?

## Continuation

Use this cycle's v003 as the parent. Probe the `small`/ghost split together: for example, `small` at a fortieth, or the ghost moved back up while `small` stays at a twentieth. Separately, port v002's `MAX_FREQ = 7` onto v003 to test whether the two stack. Add tests at capacities large enough to exercise the new ratio.

Stop probing ghost size on its own. Before chasing more gains this small, get a second cache size measured. Still pending: porting the fingerprint ring, and measuring `metadata-size` and `per-op-time`.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
